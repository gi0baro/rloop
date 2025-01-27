import asyncio


# import socket

# import pytest


class DatagramProto(asyncio.DatagramProtocol):
    done = None

    def __init__(self, create_future=False, loop=None):
        self.state = 'INITIAL'
        self.nbytes = 0
        if create_future:
            self.done = loop.create_future()

    def _assert_state(self, expected):
        if self.state != expected:
            raise AssertionError(f'state: {self.state!r}, expected: {expected!r}')

    def connection_made(self, transport):
        self.transport = transport
        self._assert_state('INITIAL')
        self.state = 'INITIALIZED'

    def datagram_received(self, data, addr):
        self._assert_state('INITIALIZED')
        self.nbytes += len(data)

    def error_received(self, exc):
        self._assert_state('INITIALIZED')

    def connection_lost(self, exc):
        self._assert_state('INITIALIZED')
        self.state = 'CLOSED'
        if self.done:
            self.done.set_result(None)


# def test_create_datagram_endpoint_sock(loop):
#     sock = socket.socket(socket.AF_INET, socket.SOCK_DGRAM)
#     sock.bind(('127.0.0.1', 0))
#     fut = loop.create_datagram_endpoint(
#         lambda: DatagramProto(create_future=True, loop=loop),
#         sock=sock)
#     transport, protocol = loop.run_until_complete(fut)
#     transport.close()
#     loop.run_until_complete(protocol.done)
#     assert protocol.state == 'CLOSED'


# @pytest.mark.skipif(not hasattr(socket, 'AF_UNIX'), reason='no UDS')
# def test_create_datagram_endpoint_sock_unix(loop):
#     fut = loop.create_datagram_endpoint(
#         lambda: DatagramProto(create_future=True, loop=loop),
#         family=socket.AF_UNIX)
#     transport, protocol = loop.run_until_complete(fut)
#     assert transport._sock.family == socket.AF_UNIX
#     transport.close()
#     loop.run_until_complete(protocol.done)
#     assert protocol.state == 'CLOSED'


# def test_create_datagram_endpoint_existing_sock_unix(loop):
#     with _unix_socket_path() as path:
#         sock = socket.socket(socket.AF_UNIX, type=socket.SOCK_DGRAM)
#         sock.bind(path)
#         sock.close()

#         coro = loop.create_datagram_endpoint(
#             lambda: DatagramProto(create_future=True, loop=loop),
#             path, family=socket.AF_UNIX)
#         transport, protocol = loop.run_until_complete(coro)
#         transport.close()
#         loop.run_until_complete(protocol.done)
