import pytest

import rloop


@pytest.fixture(scope='function')
def loop():
    return rloop.new_event_loop()
