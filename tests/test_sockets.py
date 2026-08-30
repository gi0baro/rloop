import asyncio
import socket
import time


_SIZE = 1024 * 1024


async def _recv_all(loop, sock, nbytes):
    buf = b''
    while len(buf) < nbytes:
        buf += await loop.sock_recv(sock, nbytes - len(buf))
    return buf


def test_socket_accept_recv(loop):
    async def server():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            sock.listen()

            fut = loop.run_in_executor(None, client, sock.getsockname())

            client_sock, _ = await loop.sock_accept(sock)
            with client_sock:
                data = await _recv_all(loop, client_sock, _SIZE)

            await fut

        return data

    def client(addr):
        sock = socket.socket()
        with sock:
            sock.connect(addr)
            sock.sendall(b'a' * _SIZE)

    data = loop.run_until_complete(server())
    assert data == b'a' * _SIZE


def test_socket_accept_send(loop):
    state = {'data': b''}

    async def server():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            sock.listen()

            fut = loop.run_in_executor(None, client, sock.getsockname())

            client_sock, _ = await loop.sock_accept(sock)
            with client_sock:
                await loop.sock_sendall(client_sock, b'a' * _SIZE)

            await fut

    def client(addr):
        sock = socket.socket()
        with sock:
            sock.connect(addr)
            while len(state['data']) < _SIZE:
                state['data'] += sock.recv(1024 * 16)

    loop.run_until_complete(server())
    assert state['data'] == b'a' * _SIZE


def test_socket_recv_cancel(loop):
    async def server():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            sock.listen()

            fut = loop.run_in_executor(None, client, sock.getsockname())

            client_sock, _ = await loop.sock_accept(sock)
            with client_sock:
                task = loop.create_task(loop.sock_recv(client_sock, 1024))
                await asyncio.sleep(0.1)
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass
                assert task.cancelled()
                data = await loop.sock_recv(client_sock, 1024)

            await fut

        return data

    def client(addr):
        sock = socket.socket()
        with sock:
            sock.connect(addr)
            time.sleep(0.3)
            sock.sendall(b'x')

    data = loop.run_until_complete(server())
    assert data == b'x'


def test_add_reader_level_triggered(loop):
    r, w = socket.socketpair()
    r.setblocking(False)
    received = bytearray()

    async def main():
        done = loop.create_future()

        def on_readable():
            received.extend(r.recv(1))
            if len(received) == 10:
                loop.remove_reader(r.fileno())
                done.set_result(None)

        loop.add_reader(r.fileno(), on_readable)
        w.send(b'x' * 10)
        await asyncio.wait_for(done, timeout=5)

    try:
        loop.run_until_complete(main())
    finally:
        r.close()
        w.close()
    assert bytes(received) == b'x' * 10


def test_add_writer_level_triggered(loop):
    r, w = socket.socketpair()
    w.setblocking(False)
    state = {'count': 0}

    async def main():
        done = loop.create_future()

        def on_writable():
            state['count'] += 1
            if state['count'] == 3:
                loop.remove_writer(w.fileno())
                done.set_result(None)

        loop.add_writer(w.fileno(), on_writable)
        await asyncio.wait_for(done, timeout=5)

    try:
        loop.run_until_complete(main())
    finally:
        r.close()
        w.close()
    assert state['count'] == 3


def test_socket_op_fd_recycle(loop):
    async def parked_recv(sock, peer, payload):
        task = loop.create_task(loop.sock_recv(sock, 1024))
        loop.call_later(0.05, peer.send, payload)
        return await asyncio.wait_for(task, timeout=5)

    async def main():
        a1, b1 = socket.socketpair()
        a1.setblocking(False)
        try:
            assert await parked_recv(a1, b1, b'x') == b'x'
        finally:
            a1.close()
            b1.close()

        a2, b2 = socket.socketpair()
        a2.setblocking(False)
        try:
            assert await parked_recv(a2, b2, b'y') == b'y'
        finally:
            a2.close()
            b2.close()

    loop.run_until_complete(main())
