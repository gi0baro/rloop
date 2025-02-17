import socket

from . import BaseProto


class ProtoServer(BaseProto):
    def data_received(self, data):
        super().data_received(data)
        self.data += data
        self.transport.write(b'hello back')
        self.transport.write_eof()


class ProtoClient(BaseProto):
    def connection_made(self, transport):
        super().connection_made(transport)
        transport.write(b'hello')
        transport.write_eof()

    def data_received(self, data):
        super().data_received(data)
        self.data += data


def test_tcp_connection_send(loop):
    proto_srv = ProtoServer()
    proto_cli = ProtoClient(loop.create_future)

    async def main():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            addr = sock.getsockname()
            srv = await loop.create_server(lambda: proto_srv, sock=sock)
            _ = await loop.create_connection(lambda: proto_cli, *addr)
            await proto_cli._done
            srv.close()

    loop.run_until_complete(main())
    assert proto_cli.state == 'CLOSED'
    assert proto_srv.state == 'CLOSED'
    assert proto_srv.data == b'hello'
    assert proto_cli.data == b'hello back'
