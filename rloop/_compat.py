import sys


_PYV = int(sys.version_info.major * 100 + sys.version_info.minor)
_PY_311 = 311
_PY_314 = 314

if _PYV < 314:
    from asyncio.events import BaseDefaultEventLoopPolicy as _BaseEventLoopPolicy
else:
    from asyncio.events import _BaseDefaultEventLoopPolicy as _BaseEventLoopPolicy  # noqa
