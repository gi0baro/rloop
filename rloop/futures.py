from asyncio.futures import Future as _Future


class _SyncSockReaderFuture(_Future):
    def __init__(self, sock, loop):
        super().__init__(loop=loop)
        self.__sock = sock

    def cancel(self, msg=None) -> bool:
        if self.__sock is not None and self.__sock.fileno() != -1:
            self._loop.remove_reader(self.__sock)
            self.__sock = None
        return super().cancel(msg)


class _SyncSockWriterFuture(_Future):
    def __init__(self, sock, loop):
        super().__init__(loop=loop)
        self.__sock = sock

    def cancel(self, msg=None) -> bool:
        if self.__sock is not None and self.__sock.fileno() != -1:
            self._loop.remove_writer(self.__sock)
            self.__sock = None
        return super().cancel(msg)
