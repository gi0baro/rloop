import asyncio as __asyncio
import socket
import subprocess
import sys
import threading
import traceback
import warnings
from asyncio.events import _get_running_loop, _set_running_loop
from asyncio.futures import Future as _Future, isfuture as _isfuture
from asyncio.log import logger as _aio_logger
from asyncio.tasks import Task as _Task, ensure_future as _ensure_future, gather as _gather
from contextvars import copy_context as _copy_context
from typing import Union

from ._rloop import CBHandle, EventLoop as __BaseLoop, TimerHandle
from .futures import _SyncSockReaderFuture, _SyncSockWriterFuture
from .utils import _HAS_IPv6, _ipaddr_info


class RLoop(__BaseLoop, __asyncio.AbstractEventLoop):
    #: running methods
    def run_forever(self):
        try:
            _old_agen_hooks = self._run_forever_pre()
            self._run()
        finally:
            self._run_forever_post(_old_agen_hooks)

    def _run_forever_pre(self):
        self._check_closed()
        self._check_running()
        # self._set_coroutine_origin_tracking(self._debug)

        _old_agen_hooks = sys.get_asyncgen_hooks()
        self._thread_id = threading.get_ident()
        sys.set_asyncgen_hooks(firstiter=self._asyncgen_firstiter_hook, finalizer=self._asyncgen_finalizer_hook)

        _set_running_loop(self)

        return _old_agen_hooks

    def _run_forever_post(self, _old_agen_hooks):
        self._stopping = False
        self._thread_id = 0
        _set_running_loop(None)
        # self._set_coroutine_origin_tracking(False)
        # Restore any pre-existing async generator hooks.
        if _old_agen_hooks is not None:
            sys.set_asyncgen_hooks(*_old_agen_hooks)
            self._old_agen_hooks = None

    def run_until_complete(self, future):
        self._check_closed()
        self._check_running()

        new_task = not _isfuture(future)
        future = _ensure_future(future, loop=self)
        if new_task:
            # An exception is raised if the future didn't complete, so there
            # is no need to log the "destroy pending task" message
            future._log_destroy_pending = False

        future.add_done_callback(self._run_until_complete_cb)
        try:
            self.run_forever()
        except:
            if new_task and future.done() and not future.cancelled():
                # The coroutine raised a BaseException. Consume the exception
                # to not log a warning, the caller doesn't have access to the
                # local task.
                future.exception()
            raise
        finally:
            future.remove_done_callback(self._run_until_complete_cb)
        if not future.done():
            raise RuntimeError('Event loop stopped before Future completed.')

        return future.result()

    def _run_until_complete_cb(self, fut):
        if not fut.cancelled():
            exc = fut.exception()
            if isinstance(exc, (SystemExit, KeyboardInterrupt)):
                # Issue #336: run_forever() already finished,
                # no need to stop it.
                return
        self.stop()

    def stop(self):
        self._stopping = True

    def _check_running(self):
        if self.is_running():
            raise RuntimeError('This event loop is already running')
        if _get_running_loop() is not None:
            raise RuntimeError('Cannot run the event loop while another loop is running')

    def is_running(self) -> bool:
        return bool(self._thread_id)

    def _check_closed(self):
        if self._closed:
            raise RuntimeError('Event loop is closed')

    def is_closed(self) -> bool:
        return self._closed

    def close(self):
        if self.is_running():
            raise RuntimeError('Cannot close a running event loop')
        if self._closed:
            return
        # if self._debug:
        #     logger.debug("Close %r", self)
        self._closed = True
        self._executor_shutdown_called = True
        executor = self._default_executor
        if executor is not None:
            # self._default_executor = None
            # executor.shutdown(wait=False)
            raise NotImplementedError

    async def shutdown_asyncgens(self):
        self._asyncgens_shutdown_called = True

        if not len(self._asyncgens):
            return

        closing_agens = list(self._asyncgens)
        self._asyncgens.clear()

        results = await _gather(*[ag.aclose() for ag in closing_agens], return_exceptions=True)

        for result, agen in zip(results, closing_agens):
            if isinstance(result, Exception):
                self.call_exception_handler(
                    {
                        'message': f'an error occurred during closing of asynchronous generator {agen!r}',
                        'exception': result,
                        'asyncgen': agen,
                    }
                )

    def _asyncgen_finalizer_hook(self, agen):
        self._asyncgens.discard(agen)
        if not self.is_closed():
            self.call_soon_threadsafe(self.create_task, agen.aclose())

    def _asyncgen_firstiter_hook(self, agen):
        if self._asyncgens_shutdown_called:
            warnings.warn(  # noqa: B028
                f'asynchronous generator {agen!r} was scheduled after loop.shutdown_asyncgens() call',
                ResourceWarning,
                source=self,
            )

        self._asyncgens.add(agen)

    # TODO
    async def shutdown_default_executor(self, timeout=None):
        if self._default_executor is None:
            return

        raise NotImplementedError

    #: callback scheduling methods
    # def _timer_handle_cancelled(self, handle):
    #     raise NotImplementedError

    def call_soon(self, callback, *args, context=None) -> CBHandle:
        return self._call_soon(callback, args, context or _copy_context())

    def call_later(self, delay, callback, *args, context=None) -> Union[CBHandle, TimerHandle]:
        if delay <= 0:
            return self._call_soon(callback, args, context or _copy_context())
        delay = round(delay * 1_000_000)
        return self._call_later(delay, callback, args, context or _copy_context())

    def call_at(self, when, callback, *args, context=None) -> Union[CBHandle, TimerHandle]:
        delay = when - self.time()
        if delay <= 0:
            return self._call_soon(callback, args, context or _copy_context())
        return self._call_later(delay, callback, args, context or _copy_context())

    def time(self) -> float:
        return self._clock / 1_000_000

    def create_future(self) -> _Future:
        return _Future(loop=self)

    def create_task(self, coro, *, name=None, context=None) -> _Task:
        self._check_closed()
        if self._task_factory is None:
            task = _Task(coro, loop=self, context=context)
            if task._source_traceback:
                del task._source_traceback[-1]
        else:
            if context is None:
                # Use legacy API if context is not needed
                task = self._task_factory(self, coro)
            else:
                task = self._task_factory(self, coro, context=context)

            task.set_name(name)

        return task

    #: threads methods
    def call_soon_threadsafe(self, callback, *args, context=None) -> CBHandle:
        return self._call_soon(callback, args, context or self._base_ctx)

    # TODO
    def run_in_executor(self, executor, func, *args):
        raise NotImplementedError

    # TODO
    def set_default_executor(self, executor):
        raise NotImplementedError

    #: network I/O methods
    async def getaddrinfo(self, host, port, *, family=0, type=0, proto=0, flags=0):
        raise NotImplementedError

    async def getnameinfo(self, sockaddr, flags=0):
        raise NotImplementedError

    async def create_connection(
        self,
        protocol_factory,
        host=None,
        port=None,
        *,
        ssl=None,
        family=0,
        proto=0,
        flags=0,
        sock=None,
        local_addr=None,
        server_hostname=None,
        ssl_handshake_timeout=None,
        ssl_shutdown_timeout=None,
        happy_eyeballs_delay=None,
        interleave=None,
    ):
        raise NotImplementedError

    async def create_server(
        self,
        protocol_factory,
        host=None,
        port=None,
        *,
        family=socket.AF_UNSPEC,
        flags=socket.AI_PASSIVE,
        sock=None,
        backlog=100,
        ssl=None,
        reuse_address=None,
        reuse_port=None,
        keep_alive=None,
        ssl_handshake_timeout=None,
        ssl_shutdown_timeout=None,
        start_serving=True,
    ):
        raise NotImplementedError

    async def sendfile(self, transport, file, offset=0, count=None, *, fallback=True):
        raise NotImplementedError

    async def start_tls(
        self,
        transport,
        protocol,
        sslcontext,
        *,
        server_side=False,
        server_hostname=None,
        ssl_handshake_timeout=None,
        ssl_shutdown_timeout=None,
    ):
        raise NotImplementedError

    async def create_unix_connection(
        self,
        protocol_factory,
        path=None,
        *,
        ssl=None,
        sock=None,
        server_hostname=None,
        ssl_handshake_timeout=None,
        ssl_shutdown_timeout=None,
    ):
        raise NotImplementedError

    async def create_unix_server(
        self,
        protocol_factory,
        path=None,
        *,
        sock=None,
        backlog=100,
        ssl=None,
        ssl_handshake_timeout=None,
        ssl_shutdown_timeout=None,
        start_serving=True,
    ):
        raise NotImplementedError

    async def connect_accepted_socket(
        self, protocol_factory, sock, *, ssl=None, ssl_handshake_timeout=None, ssl_shutdown_timeout=None
    ):
        raise NotImplementedError

    async def create_datagram_endpoint(
        self,
        protocol_factory,
        local_addr=None,
        remote_addr=None,
        *,
        family=0,
        proto=0,
        flags=0,
        reuse_address=None,
        reuse_port=None,
        allow_broadcast=None,
        sock=None,
    ):
        raise NotImplementedError

    #: pipes and subprocesses methods
    async def connect_read_pipe(self, protocol_factory, pipe):
        raise NotImplementedError

    async def connect_write_pipe(self, protocol_factory, pipe):
        raise NotImplementedError

    async def subprocess_shell(
        self, protocol_factory, cmd, *, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs
    ):
        raise NotImplementedError

    async def subprocess_exec(
        self, protocol_factory, *args, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE, **kwargs
    ):
        raise NotImplementedError

    #: ready-based callback registration methods
    def add_reader(self, fd, callback, *args) -> CBHandle:
        return self._reader_add(fd, callback, args, _copy_context())

    def remove_reader(self, fd) -> bool:
        return self._reader_rem(fd)

    def add_writer(self, fd, callback, *args) -> CBHandle:
        return self._writer_add(fd, callback, args, _copy_context())

    def remove_writer(self, fd) -> bool:
        return self._writer_rem(fd)

    #: completion based I/O methods
    def sock_recv(self, sock, nbytes) -> _Future:
        future = _SyncSockReaderFuture(sock, self)
        self._reader_add(sock.fileno(), self._sock_recv, (future, sock, nbytes), _copy_context())
        return future

    def _sock_recv(self, fut, sock, n):
        try:
            data = sock.recv(n)
        except (BlockingIOError, InterruptedError):
            return
        except (SystemExit, KeyboardInterrupt):
            raise
        except BaseException as exc:
            fut.set_exception(exc)
            self._reader_rem(sock.fileno())
        else:
            fut.set_result(data)
            self._reader_rem(sock.fileno())

    def sock_recv_into(self, sock, buf) -> _Future:
        future = _SyncSockReaderFuture(sock, self)
        self._reader_add(sock.fileno(), self._sock_recv_into, (future, sock, buf), _copy_context())
        return future

    def _sock_recv_into(self, fut, sock, buf):
        try:
            data = sock.recv_into(buf)
        except (BlockingIOError, InterruptedError):
            return
        except (KeyboardInterrupt, SystemExit):
            raise
        except BaseException as exc:
            fut.set_exception(exc)
            self._reader_rem(sock.fileno())
        else:
            fut.set_result(data)
            self._reader_rem(sock.fileno())

    # async def sock_recvfrom(self, sock, bufsize):
    #     raise NotImplementedError

    # async def sock_recvfrom_into(self, sock, buf, nbytes=0):
    #     raise NotImplementedError

    async def sock_sendall(self, sock, data):
        if not data:
            return

        try:
            n = sock.send(data)
        except (BlockingIOError, InterruptedError):
            pass
        else:
            if n == len(data):
                return
            data = memoryview(data)
            data = data[n:]

        future = _SyncSockWriterFuture(sock, self)
        self._writer_add(sock.fileno(), self._sock_sendall, (future, sock, data), _copy_context())
        return await future

    def _sock_sendall(self, fut, sock, data):
        try:
            n = sock.send(data)
        except (BlockingIOError, InterruptedError):
            return
        except (SystemExit, KeyboardInterrupt):
            raise
        except BaseException as exc:
            fut.set_exception(exc)
            self._writer_rem(sock.fileno())
            return

        self._writer_rem(sock.fileno())
        if n == len(data):
            fut.set_result(None)
        else:
            data = data[n:]
            self._writer_add(sock.fileno(), self._sock_sendall, (fut, sock, data), _copy_context())

    # async def sock_sendto(self, sock, data, address):
    #     raise NotImplementedError

    async def sock_connect(self, sock, address):
        if sock.family == socket.AF_INET or (_HAS_IPv6 and sock.family == socket.AF_INET6):
            resolved = await self._ensure_resolved(
                address,
                family=sock.family,
                type=sock.type,
                proto=sock.proto,
            )
            _, _, _, _, address = resolved[0]

        fut = self._sock_connect(sock, address)
        if fut is not None:
            await fut

    async def _ensure_resolved(self, address, *, family=0, type=socket.SOCK_STREAM, proto=0, flags=0):
        host, port = address[:2]
        info = _ipaddr_info(host, port, family, type, proto, *address[2:])
        if info is not None:
            # "host" is already a resolved IP.
            return [info]
        else:
            return await self.getaddrinfo(host, port, family=family, type=type, proto=proto, flags=flags)

    def _sock_connect(self, sock, address) -> _Future:
        try:
            sock.connect(address)
        except (BlockingIOError, InterruptedError):
            pass
        else:
            return

        future = _SyncSockWriterFuture(sock, self)
        self._writer_add(sock.fileno(), self._sock_connect_cb, (future, sock, address), _copy_context())
        return future

    def _sock_connect_cb(self, fut, sock, address):
        try:
            err = sock.getsockopt(socket.SOL_SOCKET, socket.SO_ERROR)
            if err != 0:
                # Jump to any except clause below.
                raise OSError(err, 'Connect call failed %s' % (address,))
        except (BlockingIOError, InterruptedError):
            return
        except (KeyboardInterrupt, SystemExit):
            raise
        except BaseException as exc:
            fut.set_exception(exc)
            self._writer_rem(sock.fileno())
        else:
            fut.set_result(None)
            self._writer_rem(sock.fileno())

    def sock_accept(self, sock) -> _Future:
        future = _SyncSockReaderFuture(sock, self)
        self._reader_add(sock.fileno(), self._sock_accept, (future, sock), _copy_context())
        return future

    def _sock_accept(self, fut, sock):
        try:
            conn, address = sock.accept()
            conn.setblocking(False)
        except (BlockingIOError, InterruptedError):
            return
        except (SystemExit, KeyboardInterrupt):
            raise
        except BaseException as exc:
            fut.set_exception(exc)
            self._reader_rem(sock.fileno())
        else:
            fut.set_result((conn, address))
            self._reader_rem(sock.fileno())

    # async def sock_sendfile(self, sock, file, offset=0, count=None, *, fallback=None):
    #     raise NotImplementedError

    #: signals
    def add_signal_handler(self, sig, callback, *args):
        raise NotImplementedError

    def remove_signal_handler(self, sig):
        raise NotImplementedError

    #: task factory
    def set_task_factory(self, factory):
        self._task_factory = factory

    def get_task_factory(self):
        return self._task_factory

    #: error handlers
    def get_exception_handler(self):
        return self._exception_handler

    def set_exception_handler(self, handler):
        self._exception_handler = handler

    def default_exception_handler(self, context):
        message = context.get('message')
        if not message:
            message = 'Unhandled exception in event loop'

        exception = context.get('exception')
        if exception is not None:
            exc_info = (type(exception), exception, exception.__traceback__)
        else:
            exc_info = False

        # if ('source_traceback' not in context and
        #         self._current_handle is not None and
        #         self._current_handle._source_traceback):
        #     context['handle_traceback'] = \
        #         self._current_handle._source_traceback

        log_lines = [message]
        for key in sorted(context):
            if key in {'message', 'exception'}:
                continue
            value = context[key]
            if key == 'source_traceback':
                tb = ''.join(traceback.format_list(value))
                value = 'Object created at (most recent call last):\n'
                value += tb.rstrip()
            elif key == 'handle_traceback':
                tb = ''.join(traceback.format_list(value))
                value = 'Handle created at (most recent call last):\n'
                value += tb.rstrip()
            else:
                value = repr(value)
            log_lines.append(f'{key}: {value}')

        _aio_logger.error('\n'.join(log_lines), exc_info=exc_info)

    def call_exception_handler(self, context):
        if self._exception_handler is None:
            try:
                self.default_exception_handler(context)
            except (KeyboardInterrupt, SystemExit):
                raise
            except BaseException:
                _aio_logger.error('Exception in default exception handler', exc_info=True)
        else:
            try:
                self._exception_handler(self, context)
            except (KeyboardInterrupt, SystemExit):
                raise
            except BaseException as exc:
                # Exception in the user set custom exception handler.
                try:
                    # Let's try default handler.
                    self.default_exception_handler(
                        {
                            'message': 'Unhandled error in exception handler',
                            'exception': exc,
                            'context': context,
                        }
                    )
                except (KeyboardInterrupt, SystemExit):
                    raise
                except BaseException:
                    # Guard 'default_exception_handler' in case it is
                    # overloaded.
                    _aio_logger.error(
                        'Exception in default exception handler '
                        'while handling an unexpected error '
                        'in custom exception handler',
                        exc_info=True,
                    )

    #: debug management
    def get_debug(self) -> bool:
        return False

    # TODO
    def set_debug(self, enabled: bool):
        return
