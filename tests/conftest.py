import asyncio

import pytest

import rloop


EVENT_LOOPS = [
    asyncio.new_event_loop,
    rloop.new_event_loop,
]


@pytest.fixture(scope='function', params=EVENT_LOOPS, ids=lambda x: type(x()))
def loop(request):
    return request.param()
