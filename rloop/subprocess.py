import itertools
import os
import subprocess
import threading
import warnings
from asyncio import base_subprocess as _base_subp, transports as _transports

from .exc import _aio_logger
from .transports import _TransportFlowControl


class _ChildWatcher:
    __slots__ = ['loop']

    def __init__(self, loop):
        self._loop = loop

    @staticmethod
    def _waitstatus_to_exitcode(status):
        try:
            return os.waitstatus_to_exitcode(status)
        except ValueError:
            return status


class _PidfdChildWatcher(_ChildWatcher):
    def add_child_handler(self, pid, callback, *args):
        pidfd = os.pidfd_open(pid)
        self._loop.add_reader(pidfd, self._do_wait, pid, pidfd, callback, args)

    def _do_wait(self, pid, pidfd, callback, args):
        self._loop._remove_reader(pidfd)
        try:
            _, status = os.waitpid(pid, 0)
        except ChildProcessError:
            # The child process is already reaped
            # (may happen if waitpid() is called elsewhere).
            returncode = 255
            _aio_logger.warning('child process pid %d exit status already read: will report returncode 255', pid)
        else:
            returncode = self._waitstatus_to_exitcode(status)

        os.close(pidfd)
        callback(pid, returncode, *args)


class _ThreadedChildWatcher(_ChildWatcher):
    def __init__(self, loop):
        super().__init__(loop)
        self._pid_counter = itertools.count(0)
        self._threads = {}

    def __del__(self, _warn=warnings.warn):
        threads = [thread for thread in list(self._threads.values()) if thread.is_alive()]
        if threads:
            _warn(f'{self.__class__} has registered but not finished child processes', ResourceWarning, source=self)

    def add_child_handler(self, pid, callback, *args):
        thread = threading.Thread(
            target=self._do_waitpid,
            name=f'asyncio-waitpid-{next(self._pid_counter)}',
            args=(self._loop, pid, callback, args),
            daemon=True,
        )
        self._threads[pid] = thread
        thread.start()

    def _do_waitpid(self, loop, expected_pid, callback, args):
        assert expected_pid > 0

        try:
            pid, status = os.waitpid(expected_pid, 0)
        except ChildProcessError:
            # The child process is already reaped
            # (may happen if waitpid() is called elsewhere).
            pid = expected_pid
            returncode = 255
            _aio_logger.warning('Unknown child process pid %d, will report returncode 255', pid)
        else:
            returncode = self._waitstatus_to_exitcode(status)
            if loop.get_debug():
                _aio_logger.debug('process %s exited with returncode %s', expected_pid, returncode)

        if loop.is_closed():
            _aio_logger.warning('Loop %r that handles pid %r is closed', loop, pid)
        else:
            loop.call_soon_threadsafe(callback, pid, returncode, *args)

        self._threads.pop(expected_pid)


class _SubProcessTransport(_base_subp.BaseSubprocessTransport):
    def _start(self, args, shell, stdin, stdout, stderr, bufsize, **kwargs):
        self._proc = subprocess.Popen(  # noqa: S603
            args,
            shell=shell,
            stdin=stdin,
            stdout=stdout,
            stderr=stderr,
            universal_newlines=False,
            bufsize=bufsize,
            **kwargs,
        )


class _PipeReadTransport(_transports.ReadTransport):
    max_size = 256 * 1024

    def __init__(self, loop, pipe, protocol, waiter, extra=None):
        self._loop = loop
        self._pipe = pipe
        self._fileno = pipe.fileno()
        self._protocol = protocol
        self._closing = False
        self._paused = False

        os.set_blocking(self._fileno, False)

        self._loop.call_soon(self._protocol.connection_made, self)
        # only start reading when connection_made() has been called
        self._loop.call_soon(self._add_reader, self._fileno, self._read_ready)
        # only wake up the waiter when connection_made() has been called
        self._loop.call_soon(self._connection_made_waiter_cb, waiter, None)

    @staticmethod
    def _connection_made_waiter_cb(fut, result):
        if fut.cancelled():
            return
        fut.set_result(result)

    def _add_reader(self, fd, callback):
        if not self.is_reading():
            return
        self._loop.add_reader(fd, callback)

    def is_reading(self):
        return not self._paused and not self._closing

    def _read_ready(self):
        try:
            data = os.read(self._fileno, self.max_size)
        except (BlockingIOError, InterruptedError):
            pass
        except OSError as exc:
            self._fatal_error(exc, 'Fatal read error on pipe transport')
        else:
            if data:
                self._protocol.data_received(data)
            else:
                self._closing = True
                self._loop.remove_reader(self._fileno)
                self._loop.call_soon(self._protocol.eof_received)
                self._loop.call_soon(self._call_connection_lost, None)

    def pause_reading(self):
        if not self.is_reading():
            return
        self._paused = True
        self._loop.remove_reader(self._fileno)

    def resume_reading(self):
        if self._closing or not self._paused:
            return
        self._paused = False
        self._loop.add_reader(self._fileno, self._read_ready)

    def set_protocol(self, protocol):
        self._protocol = protocol

    def get_protocol(self):
        return self._protocol

    def is_closing(self):
        return self._closing

    def close(self):
        if not self._closing:
            self._close(None)

    def __del__(self, _warn=warnings.warn):
        if self._pipe is not None:
            _warn(f'unclosed transport {self!r}', ResourceWarning, source=self)
            self._pipe.close()

    def _fatal_error(self, exc, message='Fatal error on pipe transport'):
        self._loop.call_exception_handler(
            {
                'message': message,
                'exception': exc,
                'transport': self,
                'protocol': self._protocol,
            }
        )
        self._close(exc)

    def _close(self, exc):
        self._closing = True
        self._loop.remove_reader(self._fileno)
        self._loop.call_soon(self._call_connection_lost, exc)

    def _call_connection_lost(self, exc):
        try:
            self._protocol.connection_lost(exc)
        finally:
            self._pipe.close()
            self._pipe = None
            self._protocol = None
            self._loop = None


class _PipeWriteTransport(_TransportFlowControl, _transports.WriteTransport):
    def __init__(self, loop, pipe, protocol, waiter, extra=None):
        super().__init__(loop)
        self._pipe = pipe
        self._fileno = pipe.fileno()
        self._protocol = protocol
        self._buffer = bytearray()
        self._conn_lost = 0
        self._closing = False  # Set when close() or write_eof() called.

        os.set_blocking(self._fileno, False)

        self._loop.call_soon(self._protocol.connection_made, self)
        # only start reading when connection_made() has been called
        self._loop.call_soon(self._add_reader, self._fileno, self._read_ready)
        # only wake up the waiter when connection_made() has been called
        self._loop.call_soon(self._connection_made_waiter_cb, waiter, None)

    @staticmethod
    def _connection_made_waiter_cb(fut, result):
        if fut.cancelled():
            return
        fut.set_result(result)

    def _add_reader(self, fd, callback):
        self._loop.add_reader(fd, callback)

    def get_write_buffer_size(self):
        return len(self._buffer)

    def _read_ready(self):
        # Pipe was closed by peer.
        if self._buffer:
            self._close(BrokenPipeError())
        else:
            self._close()

    def write(self, data):
        if isinstance(data, bytearray):
            data = memoryview(data)
        if not data:
            return

        if self._conn_lost or self._closing:
            self._conn_lost += 1
            return

        if not self._buffer:
            # Attempt to send it right away first.
            try:
                n = os.write(self._fileno, data)
            except (BlockingIOError, InterruptedError):
                n = 0
            except (SystemExit, KeyboardInterrupt):
                raise
            except BaseException as exc:
                self._conn_lost += 1
                self._fatal_error(exc, 'Fatal write error on pipe transport')
                return
            if n == len(data):
                return
            elif n > 0:
                data = memoryview(data)[n:]
            self._loop.add_writer(self._fileno, self._write_ready)

        self._buffer += data
        self._maybe_pause_protocol()

    def _write_ready(self):
        assert self._buffer, 'Data should not be empty'

        try:
            n = os.write(self._fileno, self._buffer)
        except (BlockingIOError, InterruptedError):
            pass
        except (SystemExit, KeyboardInterrupt):
            raise
        except BaseException as exc:
            self._buffer.clear()
            self._conn_lost += 1
            # Remove writer here, _fatal_error() doesn't it
            # because _buffer is empty.
            self._loop.remove_writer(self._fileno)
            self._fatal_error(exc, 'Fatal write error on pipe transport')
        else:
            if n == len(self._buffer):
                self._buffer.clear()
                self._loop.remove_writer(self._fileno)
                self._maybe_resume_protocol()  # May append to buffer.
                if self._closing:
                    self._loop.remove_reader(self._fileno)
                    self._call_connection_lost(None)
                return
            elif n > 0:
                del self._buffer[:n]

    def can_write_eof(self):
        return True

    def write_eof(self):
        if self._closing:
            return
        assert self._pipe
        self._closing = True
        if not self._buffer:
            self._loop.remove_reader(self._fileno)
            self._loop.call_soon(self._call_connection_lost, None)

    def set_protocol(self, protocol):
        self._protocol = protocol

    def get_protocol(self):
        return self._protocol

    def is_closing(self):
        return self._closing

    def close(self):
        if self._pipe is not None and not self._closing:
            # write_eof is all what we needed to close the write pipe
            self.write_eof()

    def __del__(self, _warn=warnings.warn):
        if self._pipe is not None:
            _warn(f'unclosed transport {self!r}', ResourceWarning, source=self)
            self._pipe.close()

    def abort(self):
        self._close(None)

    def _fatal_error(self, exc, message='Fatal error on pipe transport'):
        self._loop.call_exception_handler(
            {
                'message': message,
                'exception': exc,
                'transport': self,
                'protocol': self._protocol,
            }
        )
        self._close(exc)

    def _close(self, exc=None):
        self._closing = True
        if self._buffer:
            self._loop.remove_writer(self._fileno)
        self._buffer.clear()
        self._loop.remove_reader(self._fileno)
        self._loop.call_soon(self._call_connection_lost, exc)

    def _call_connection_lost(self, exc):
        try:
            self._protocol.connection_lost(exc)
        finally:
            self._pipe.close()
            self._pipe = None
            self._protocol = None
            self._loop = None
