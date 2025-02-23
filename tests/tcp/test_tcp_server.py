import asyncio
import errno
import socket

import pytest

from rloop.utils import _HAS_IPv6

from . import BaseProto


_SIZE = 1024 * 1000


class EchoProtocol(BaseProto):
    def data_received(self, data):
        super().data_received(data)
        self.transport.write(data)


@pytest.mark.skipif(not hasattr(socket, 'SOCK_NONBLOCK'), reason='no socket.SOCK_NONBLOCK')
def test_create_server_stream_bittype(loop):
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM | socket.SOCK_NONBLOCK)
    with sock:
        coro = loop.create_server(lambda: None, sock=sock)
        srv = loop.run_until_complete(coro)
        srv.close()
        loop.run_until_complete(srv.wait_closed())


@pytest.mark.skipif(not _HAS_IPv6, reason='no IPv6')
def test_create_server_ipv6(loop):
    async def main():
        srv = await asyncio.start_server(lambda: None, '::1', 0)
        try:
            assert len(srv.sockets) > 0
        finally:
            srv.close()
            await srv.wait_closed()

    try:
        loop.run_until_complete(main())
    except OSError as ex:
        if hasattr(errno, 'EADDRNOTAVAIL') and ex.errno == errno.EADDRNOTAVAIL:
            pass
        else:
            raise


def test_tcp_server_recv_send(loop):
    msg = b'a' * _SIZE
    state = {'data': b''}
    proto = EchoProtocol()

    async def main():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            addr = sock.getsockname()
            srv = await loop.create_server(lambda: proto, sock=sock)
            fut = loop.run_in_executor(None, client, addr)
            await fut
            srv.close()

    def client(addr):
        sock = socket.socket()
        with sock:
            sock.connect(addr)
            sock.sendall(msg)
            while len(state['data']) < _SIZE:
                state['data'] += sock.recv(1024 * 16)

    loop.run_until_complete(main())
    assert proto.state == 'CLOSED'
    assert state['data'] == msg


# TODO: test buffered proto
