from ._compat import _BaseEventLoopPolicy as __BasePolicy
from ._rloop import __version__ as __version__
from .loop import RLoop


def new_event_loop() -> RLoop:
    return RLoop()


class EventLoopPolicy(__BasePolicy):
    def _loop_factory(self) -> RLoop:
        return new_event_loop()
