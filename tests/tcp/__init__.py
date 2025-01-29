import asyncio


class BaseProto(asyncio.Protocol):
    _done = None

    def __init__(self, create_future=None):
        self.state = 'INITIAL'
        self.transport = None
        self.data = b''
        if create_future:
            self._done = create_future()

    def _assert_state(self, *expected):
        if self.state not in expected:
            raise AssertionError(f'state: {self.state!r}, expected: {expected!r}')

    def connection_made(self, transport):
        self.transport = transport
        self._assert_state('INITIAL')
        self.state = 'CONNECTED'

    def data_received(self, data):
        self._assert_state('CONNECTED')

    def eof_received(self):
        self._assert_state('CONNECTED')
        self.state = 'EOF'

    def connection_lost(self, exc):
        self._assert_state('CONNECTED', 'EOF')
        self.transport = None
        self.state = 'CLOSED'
        if self._done:
            self._done.set_result(None)
