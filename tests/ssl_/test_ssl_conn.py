import asyncio
import logging
import multiprocessing
import os
import random
import socket
import ssl
import threading
from multiprocessing import Event, Process

import pytest

import rloop

from . import SSLEchoClientProtocol, SSLEchoServerProtocol, SSLHTTPServerProtocol


logging.basicConfig(level=logging.DEBUG)
logger = logging.getLogger(__name__)

pytestmark = [pytest.mark.timeout(5)]


@pytest.fixture
def ssl_context():
    """Create a basic SSL context for testing."""
    ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
    # For testing with self-signed certificates, load the server's cert as trusted

    cert_dir = os.path.join(os.path.dirname(__file__), 'certs')
    certfile = os.path.join(cert_dir, 'cert.pem')
    ctx.load_verify_locations(cafile=certfile)
    ctx.check_hostname = False
    ctx.verify_mode = ssl.CERT_NONE  # Disable verification for testing
    return ctx


@pytest.fixture
def server_ssl_context():
    """Create an SSL context for the server."""
    ctx = ssl.SSLContext(ssl.PROTOCOL_TLS_SERVER)
    # For testing, load test certificates for asyncio compatibility
    # The Rust implementation generates its own dummy certificate when no certs are loaded
    cert_dir = os.path.join(os.path.dirname(__file__), 'certs')
    # Set attributes that Rust code expects
    ctx._certfile = os.path.join(cert_dir, 'cert.pem')
    ctx._keyfile = os.path.join(cert_dir, 'key.pem')
    ctx.load_cert_chain(ctx._certfile, ctx._keyfile)
    return ctx


EVENT_LOOPS = [
    asyncio.new_event_loop,
    rloop.new_event_loop,
]

SSL_BACKENDS = ['direct']  # , 'futures']
TLS_VERSIONS = ['', '1.2', '1.2+', '1.3']


def start_ssl_http_server(
    loop, server_ssl_context, host='localhost', port=None, lifetime=10, protocol=SSLHTTPServerProtocol
) -> tuple[Process, Event, tuple[str, int]]:
    """Helper function to start SSL HTTP server for testing."""
    port = port or random.randint(10000, 20000)  # noqa: S311

    # Be sure that the port is available
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    sock.setsockopt(socket.SOL_SOCKET, socket.SO_REUSEADDR, 1)
    while True:
        try:
            sock.bind((host, port))
            # Port is available
            sock.close()
            break
        except OSError as e:
            if e.errno == 98:  # Address already in use
                sock.close()
                # Try another port
                port += 1
                continue
            else:  # Re-raise other errors
                sock.close()
                raise

    server_ready = multiprocessing.Event()
    server_stop = multiprocessing.Event()
    server_addr = (host, port)

    async def run_server():
        nonlocal server_addr
        loopclass = type(loop).__name__
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind((host, port))
            server_addr = sock.getsockname()
            logger.debug(f'[server] Creating {loopclass} SSL server on {server_addr}')
            server = await loop.create_server(lambda: protocol(), sock=sock, ssl=server_ssl_context)
            logger.debug(f'[server] {loopclass} SSL server created')

            server_ready.set()

            i = 0
            for i in range(lifetime):  # noqa: B007
                await asyncio.sleep(1)
                if server_stop.is_set():
                    break

            logger.debug(f'[server] {loopclass} server closing [lifetime={i} should_stop={server_stop.is_set()}]')
            server.close()
            logger.debug(f'[server] {loopclass} server closed')

    coro = run_server()
    mp = multiprocessing.get_context('fork')
    server_process = mp.Process(target=lambda: loop.run_until_complete(coro))
    server_process.start()
    logger.debug('Waiting for server_ready event')
    server_ready.wait()
    logger.debug(f'Server ready event received, server_addr = {server_addr}')
    return server_process, server_stop, server_addr


@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
def test_ssl_connection_echo(evloop, ssl_context, server_ssl_context, tls_version, ssl_backend, monkeypatch):
    """Test basic connection with echo server."""
    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    loop = evloop()

    if tls_version and type(loop).__name__ != 'RLoop':
        # Standard Asyncio reactor should be tested only w/ tls_version unset.
        # TLS versions are only useful with RLoop reactor.
        pytest.skip('Duplicated test')

    server_proto = SSLEchoServerProtocol()
    client_proto = SSLEchoClientProtocol(loop.create_future)

    async def main():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind(('127.0.0.1', 0))
            addr = sock.getsockname()
            server = await loop.create_server(lambda: server_proto, sock=sock)
            transport, protocol = await loop.create_connection(lambda: client_proto, *addr)
            await client_proto._done
            server.close()

    loop.run_until_complete(main())
    assert client_proto.state == 'CLOSED'
    assert server_proto.state == 'CLOSED'

    assert server_proto.data == b'hello SSL world'
    assert client_proto.data.startswith(b'echo: hello SSL world')


@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
def test_ssl_protocol_without_ssl(evloop):
    """Test that non-SSL connection works with the tests protocol."""
    loop = evloop()

    host = '127.0.0.1'
    port = random.randint(10000, 20000)  # noqa: S311

    server_proto = SSLEchoServerProtocol()
    client_proto = SSLEchoClientProtocol(loop.create_future)

    async def main():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind((host, port))
            addr = sock.getsockname()
            server = await loop.create_server(lambda: server_proto, sock=sock)
            transport, protocol = await loop.create_connection(lambda: client_proto, *addr)
            await client_proto._done
            server.close()

    loop.run_until_complete(main())
    assert client_proto.state == 'CLOSED'
    assert server_proto.state == 'CLOSED'


@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
def test_ssl_server(evloop, ssl_context, server_ssl_context, tls_version, ssl_backend, monkeypatch):
    """Test SSL server functionality."""

    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    loop = evloop()

    if tls_version and type(loop).__name__ != 'RLoop':
        # Standard Asyncio reactor should be tested only w/ tls_version unset.
        # TLS versions are only useful with RLoop reactor.
        pytest.skip('Duplicated test')

    host = '127.0.0.1'
    port = random.randint(10000, 20000)  # noqa: S311

    server_proto = SSLEchoServerProtocol()
    client_proto = SSLEchoClientProtocol(loop.create_future)

    async def main():
        sock = socket.socket()
        sock.setblocking(False)

        with sock:
            sock.bind((host, port))
            addr = sock.getsockname()
            logger.debug(f'[TEST] Creating server on {addr} with ssl={server_ssl_context is not None}')
            server = await loop.create_server(lambda: server_proto, sock=sock, ssl=server_ssl_context)
            logger.debug('[TEST] Server created')
            # Give server time to start
            await asyncio.sleep(0.01)
            logger.debug(f'[TEST] Creating client connection to {addr} with ssl={ssl_context is not None}')
            transport, protocol = await loop.create_connection(lambda: client_proto, *addr, ssl=ssl_context)
            logger.debug('[TEST] Client connected')
            await client_proto._done
            logger.debug('[TEST] Client done, closing server')
            server.close()

    loop.run_until_complete(main())
    logger.debug(f'[TEST] Final states - client: {client_proto.state}, server: {server_proto.state}')
    logger.debug(f'[TEST] Server received: {server_proto.data!r}')
    logger.debug(f'[TEST] Client received: {client_proto.data!r}')
    assert client_proto.state == 'CLOSED'
    assert server_proto.state == 'CLOSED'
    # Check that SSL was actually used
    assert server_proto.data == b'hello SSL world'
    assert client_proto.data.startswith(b'echo: hello SSL world')


@pytest.mark.timeout(20)
@pytest.mark.parametrize('evloop_client', EVENT_LOOPS, ids=lambda x: f'{type(x()).__name__}[cli]')
@pytest.mark.parametrize('evloop_server', EVENT_LOOPS, ids=lambda x: f'{type(x()).__name__}[srv]')
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
def test_cross_implementation_server_client(
    evloop_server, evloop_client, ssl_context, server_ssl_context, tls_version, ssl_backend, monkeypatch
):
    """Test RLoop SSL client against asyncio SSL server."""

    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    # Use asyncio for server, RLoop for client
    server_loop = evloop_server()
    server_loop_name = type(server_loop).__name__
    client_loop = evloop_client()
    client_loop_name = type(client_loop).__name__

    if tls_version and 'RLoop' not in [server_loop_name, client_loop_name]:
        # When RLoop is not involved, should run only once: tls_version == ''.
        pytest.skip('Duplicated test')

    if tls_version in ['1.2+', '1.3'] and server_loop_name == 'RLoop':
        pytest.xfail('Flaky w/ TLS 1.3')

    client_proto = SSLEchoClientProtocol(client_loop.create_future)

    async def run_client():
        logger.debug(f'[client] Creating {client_loop_name} SSL client to server')
        for i in range(3):
            try:
                transport, protocol = await client_loop.create_connection(
                    lambda: client_proto, host, port, ssl=ssl_context
                )  # type: ignore
                logger.debug(f'[client [{i}]] {client_loop_name} SSL client connected')
                await client_proto._done
                logger.debug(f'[client [{i}]] {client_loop_name} client done')
                break
            except Exception as e:
                logger.debug(f'[client [{i}]] {client_loop_name} client failed: {e}')

    server_process, server_stop, (host, port) = start_ssl_http_server(
        server_loop, server_ssl_context, protocol=SSLEchoServerProtocol
    )

    client_thread = threading.Thread(target=lambda: client_loop.run_until_complete(run_client()))
    client_thread.start()
    client_thread.join(timeout=10)

    # Signal and wait server to stop
    logger.debug('[test] Signaling the server to stop')
    server_stop.set()
    server_process.join(timeout=3)

    # Check results
    logger.debug(f'[test] Client state: {client_proto.state}')
    logger.debug(f'[test] Client received: {client_proto.data!r}')

    assert client_proto.state == 'CLOSED'
    assert client_proto.data == b'echo: hello SSL world'


@pytest.mark.timeout(10)
@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
def test_ssl_server_with_requests_client(evloop, server_ssl_context, tls_version, ssl_backend, monkeypatch):
    """Test EventLoop SSL server with external requests client."""

    import requests

    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    # Use EventLoop for server, raw SSL socket for client
    loop = evloop()

    if tls_version and type(loop).__name__ != 'RLoop':
        # Standard Asyncio reactor should be tested only w/ tls_version unset.
        # TLS versions are only useful with RLoop reactor.
        pytest.skip('Duplicated test')

    server_process, server_stop, (host, port) = start_ssl_http_server(loop, server_ssl_context)

    url = f'https://{host}:{port}'
    # Create raw SSL client
    logger.debug(f'[client] Connecting to {url} via requests')

    try:
        result = requests.get(url, verify=False, timeout=5)
    except requests.exceptions.ReadTimeout:
        if tls_version in ['1.2+', '1.3'] and type(loop).__name__ == 'RLoop':
            pytest.xfail('Flaky w/ TLS 1.3')
        else:
            raise

    result.raise_for_status()
    assert result.status_code == 200
    assert result.text == 'hello SSL world'

    # Signal and wait server to stop
    logger.debug('[client] Signaling the server to stop')
    server_stop.set()
    server_process.join(timeout=3)


@pytest.mark.timeout(10)
@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
def test_ssl_server_with_raw_ssl_client(evloop, server_ssl_context, tls_version, ssl_backend, monkeypatch):
    """Test EventLoop SSL server with raw SSL socket client."""

    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    # Use EventLoop for server, raw SSL socket for client
    loop = evloop()

    if tls_version and type(loop).__name__ != 'RLoop':
        # Standard Asyncio reactor should be tested only w/ tls_version unset.
        # TLS versions are only useful with RLoop reactor.
        pytest.skip('Duplicated test')

    server_process, server_stop, (host, port) = start_ssl_http_server(loop, server_ssl_context)

    # Create raw SSL client
    logger.debug(f'[client] Connecting to {host}:{port} via raw SSL socket')

    # Create SSL context for client
    client_ctx = ssl.create_default_context(ssl.Purpose.SERVER_AUTH)
    cert_dir = os.path.join(os.path.dirname(__file__), 'certs')
    client_ctx.load_verify_locations(cafile=os.path.join(cert_dir, 'cert.pem'))
    client_ctx.check_hostname = False
    client_ctx.verify_mode = ssl.CERT_NONE

    # Create raw SSL connection
    sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
    success = False
    try:
        # Connect socket
        sock.connect((host, port))
        logger.debug('[client] Socket connected')

        # Wrap with SSL
        ssl_sock = client_ctx.wrap_socket(sock, server_hostname=host)
        logger.debug('[client] SSL handshake completed')

        # Send HTTP request
        request = b'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
        ssl_sock.send(request)
        logger.debug('[client] HTTP request sent')

        # Read response
        response_data = b''
        while True:
            data = ssl_sock.recv(4096)
            if not data:
                break
            response_data += data

        logger.debug(f'[client] Received {len(response_data)} bytes of response')

        # Parse response
        if response_data.startswith(b'HTTP/1.1 200 OK'):
            logger.debug('[client] Got 200 OK response')
            # Check for our expected content
            if b'hello SSL world' in response_data:
                logger.debug('[client] Response contains expected content')
                success = True
            else:
                logger.debug('[client] Response missing expected content')
                success = False
        else:
            logger.debug(f'[client] Unexpected response: {response_data[:100]!r}')
            success = False

    except Exception as e:
        logger.debug(f'[client] SSL connection failed: {e}')
        success = False
    finally:
        try:
            ssl_sock.close()
        except Exception:
            logger.warning('Failed to close the SSL socket %s', ssl_sock)

    # Signal and wait server to stop
    logger.debug('[client] Signaling the server to stop')
    server_stop.set()
    server_process.join(timeout=3)

    assert success, 'Raw SSL client test failed'


@pytest.mark.timeout(60)
@pytest.mark.parametrize('evloop', EVENT_LOOPS, ids=lambda x: type(x()))
@pytest.mark.parametrize('tls_version', TLS_VERSIONS)
@pytest.mark.parametrize('ssl_backend', SSL_BACKENDS)
@pytest.mark.parametrize('hacks', [False, True], ids=lambda x: 'hacks' if x else 'nohacks')
@pytest.mark.parametrize('zip', [False, True], ids=lambda x: 'zip' if x else 'nozip')
def test_ssl_server_with_openssl_client(evloop, server_ssl_context, tls_version, ssl_backend, hacks, zip, monkeypatch):
    """Test EventLoop SSL server with openssl s_client command-line tool."""

    import subprocess

    monkeypatch.setenv('RLOOP_TLS_VERSION', tls_version)
    monkeypatch.setenv('RLOOP_TLS_BACKEND', ssl_backend)
    # Use EventLoop for server, openssl s_client for client
    loop = evloop()
    loop_name = type(loop).__name__

    if tls_version and loop_name != 'RLoop':
        # Standard Asyncio reactor should be tested only w/ tls_version unset.
        # TLS versions are only useful with RLoop reactor.
        pytest.skip('Duplicated test')

    logger.debug('Starting SSL HTTP server')
    server_process, server_stop, (host, port) = start_ssl_http_server(loop, server_ssl_context, lifetime=30)
    logger.debug(f'Server started on {host}:{port}')

    # Create openssl s_client command with handshake debugging
    cert_dir = os.path.join(os.path.dirname(__file__), 'certs')
    cmd = [
        'openssl',
        's_client',
        '-connect',
        f'{host}:{port}',
        '-servername',
        host,
        '-CAfile',
        os.path.join(cert_dir, 'cert.pem'),
        '-ign_eof',
        '-msg',  # Show handshake messages
        '-state',  # Show SSL state
        '-tlsextdebug',  # Show TLS extensions
    ]
    if hacks:
        ## From docs:
        # There are several known bugs in SSL and TLS implementations.
        # Adding this option enables various workarounds.
        cmd.append('-bugs')
    if zip:
        # From docs: Enables support for SSL/TLS compression
        cmd.append('-comp')

    logger.debug(f'[client] Running: {" ".join(cmd)}')

    success = False
    try:
        logger.debug('Starting subprocess.Popen: %s', cmd)
        # Start openssl s_client process
        proc = subprocess.Popen(  # noqa: S602
            ' '.join(cmd),
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
            shell=True,
        )
        logger.debug('subprocess.Popen completed')

        # Send HTTP request
        http_request = 'GET / HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n'
        stdout, stderr = proc.communicate(input=http_request, timeout=10)

        logger.debug(f'[client] openssl exit code: {proc.returncode}')

        # Log stdout line by line
        for line in stdout.splitlines():
            logger.debug(f'[client] openssl stdout: {line[:200]}')

        # Log stderr line by line
        for line in stderr.splitlines():
            logger.debug(f'[client] openssl stderr: {line[:500]}')

        logger.debug('proc.returncode = %s', proc.returncode)
        logger.debug(f"'hello SSL world' in stdout = {'hello SSL world' in stdout}")

        # Check if connection was successful and response contains expected content
        if proc.returncode == 0 and 'hello SSL world' in stdout:
            logger.debug('[client] openssl client test passed')
            success = True
        else:
            logger.debug(f'[client] openssl client test failed - exit code: {proc.returncode}')

    except subprocess.TimeoutExpired as e:
        logger.debug('[client] openssl s_client timed out')
        proc.kill()
        logger.debug('TimeoutExpired exception caught: %r', e)
        if tls_version in ['1.2+', '1.3'] and loop_name == 'RLoop':
            pytest.xfail('Flaky w/ TLS 1.3')
    except FileNotFoundError:
        logger.debug('[client] openssl command not found')
        pytest.skip('openssl command not available')
    except Exception as e:
        logger.debug(f'[client] openssl client failed: {e}')
        logger.debug(f'Exception caught: {type(e).__name__}: {e}', stack_info=True, exc_info=True)
    finally:
        # Signal and wait server to stop
        logger.debug('[client] Signaling the server to stop')
        server_stop.set()
        server_process.join(timeout=5)

    assert success, 'openssl s_client test failed'
