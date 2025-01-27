import asyncio
import errno
import socket

import pytest

from rloop.utils import _HAS_IPv6


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
