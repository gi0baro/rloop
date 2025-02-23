def run_loop(loop):
    async def run():
        loop.stop()

    loop.run_until_complete(run())


def test_call_soon(loop):
    calls = []

    def cb(arg):
        calls.append(arg)

    loop.call_soon(cb, 1)
    loop.call_soon(cb, 2)
    run_loop(loop)
    assert calls == [1, 2]


def test_call_later(loop):
    calls = []

    def cb(arg):
        calls.append(arg)

    def stop():
        loop.stop()

    loop.call_later(0.001, cb, 2)
    loop.call_later(0.1, stop)
    loop.call_soon(cb, 1)
    loop.run_forever()
    assert calls == [1, 2]


def test_call_later_negative(loop):
    calls = []

    def cb(arg):
        calls.append(arg)

    loop.call_later(-1.0, cb, 1)
    loop.call_later(-2.0, cb, 2)
    run_loop(loop)
    assert calls == [1, 2]


def test_call_at(loop):
    def cb():
        loop.stop()

    delay = 0.100
    when = loop.time() + delay
    loop.call_at(when, cb)
    t0 = loop.time()
    loop.run_forever()
    dt = loop.time() - t0

    assert dt >= delay
