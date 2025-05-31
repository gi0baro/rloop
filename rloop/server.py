import asyncio
import asyncio.trsock

from ._rloop import Server as _Server


class Server:
    __slots__ = ['_inner', '_sff']

    def __init__(self, inner: _Server):
        self._inner = inner
        self._sff = None

    def get_loop(self):
        return self._inner._loop

    def is_serving(self):
        return self._inner._is_serving()

    async def start_serving(self):
        self._inner._start_serving()

    async def serve_forever(self):
        if self._sff is not None:
            raise RuntimeError(f'server {self!r} is already being awaited on serve_forever()')
        # if self._servers is None:
        #     raise RuntimeError(f'server {self!r} is closed')

        self._inner._start_serving()
        self._sff = self._inner._loop.create_future()

        try:
            await self._sff
        except asyncio.CancelledError:
            try:
                self.close()
                # await self.wait_closed()
            finally:
                raise
        finally:
            self._sff = None

    async def wait_closed(self):
        return

    # async def wait_closed(self):
    #     # if self._servers is None or self._waiters is None:
    #     #     return
    #     waiter = self._loop.create_future()
    #     self._add_waiter(waiter)
    #     # self._waiters.append(waiter)
    #     await waiter

    def close(self):
        # sockets = self._sockets
        # if sockets is None:
        #     return
        # self._sockets = None

        # for sock in sockets:
        #     self._loop._stop_serving(sock)

        # self._serving = False

        self._inner._close()

        if self._sff is not None and not self._sff.done():
            self._sff.cancel()
            self._sff = None

    def close_clients(self):
        self._inner._streams_close()

    def abort_clients(self):
        self._inner._streams_abort()

    async def __aenter__(self):
        return self

    async def __aexit__(self, *exc):
        self.close()
        # await self.wait_closed()

    # TODO
    @property
    def sockets(self):
        return tuple(asyncio.trsock.TransportSocket(s) for s in self._inner._sockets)
