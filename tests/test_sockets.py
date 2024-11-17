import socket
import threading


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

            cth = threading.Thread(target=client, args=(sock.getsockname(),))
            cth.start()

            client_sock, _ = await loop.sock_accept(sock)
            with client_sock:
                data = await _recv_all(loop, client_sock, _SIZE)

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

            cth = threading.Thread(target=client, args=(sock.getsockname(),))
            cth.start()

            client_sock, _ = await loop.sock_accept(sock)
            with client_sock:
                await loop.sock_sendall(client_sock, b'a' * _SIZE)

            cth.join()

    def client(addr):
        sock = socket.socket()
        with sock:
            sock.connect(addr)
            while len(state['data']) < _SIZE:
                state['data'] += sock.recv(1024 * 16)

    loop.run_until_complete(server())
    assert state['data'] == b'a' * _SIZE
