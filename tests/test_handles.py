import threading


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


def test_call_soon_threadsafe(loop):
    calls = []

    def cb(arg):
        calls.append(arg)

    def wake(cond):
        with cond:
            cond.notify_all()

    def stop():
        loop.stop()

    def trun(cond1, cond2, loop, cb):
        with cond1:
            cond1.wait()
        loop.call_soon_threadsafe(cb, 2)
        with cond2:
            cond2.wait()
        loop.call_soon_threadsafe(cb, 4)

    cond1 = threading.Condition()
    cond2 = threading.Condition()
    t = threading.Thread(target=trun, args=(cond1, cond2, loop, cb))
    t.start()

    loop.call_soon(cb, 1)
    loop.call_soon(wake, cond1)
    loop.call_later(0.5, cb, 3)
    loop.call_later(0.6, wake, cond2)
    loop.call_later(1.0, stop)
    loop.run_forever()

    assert calls == [1, 2, 3, 4]
