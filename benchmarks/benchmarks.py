import datetime
import json
import multiprocessing
import os
import signal
import subprocess
import sys
import time
from contextlib import contextmanager
from pathlib import Path


WD = Path(__file__).resolve().parent
CPU = multiprocessing.cpu_count()
LOOPS = ['asyncio', 'rloop', 'uvloop']
MSGS = [1024, 1024 * 10, 1024 * 100]
CONCURRENCIES = sorted({1, max(CPU / 2, 1), max(CPU - 1, 1)})


@contextmanager
def server(loop, streams=False, proto=False):
    exc_prefix = os.environ.get('BENCHMARK_EXC_PREFIX')
    py = 'python'
    if exc_prefix:
        py = f'{exc_prefix}/{py}'
    target = WD / 'server.py'
    proc_cmd = f'{py} {target} --loop {loop}'
    if streams:
        proc_cmd += ' --streams'
    if proto:
        proc_cmd += ' --proto'

    proc = subprocess.Popen(proc_cmd, shell=True, preexec_fn=os.setsid)  # noqa: S602
    time.sleep(2)
    yield proc
    os.killpg(os.getpgid(proc.pid), signal.SIGKILL)


def client(duration, concurrency, msgsize):
    exc_prefix = os.environ.get('BENCHMARK_EXC_PREFIX')
    py = 'python'
    if exc_prefix:
        py = f'{exc_prefix}/{py}'
    target = WD / 'client.py'
    cmd_parts = [
        py,
        str(target),
        f'--concurrency {concurrency}',
        f'--duration {duration}',
        f'--msize {msgsize}',
        '--output json',
    ]
    try:
        proc = subprocess.run(  # noqa: S602
            ' '.join(cmd_parts),
            shell=True,
            check=True,
            capture_output=True,
        )
        data = json.loads(proc.stdout.decode('utf8'))
        return data
    except Exception as e:
        print(f'WARN: got exception {e} while loading client data')
        return {}


def benchmark(msgs=None, concurrencies=None):
    concurrencies = concurrencies or CONCURRENCIES
    msgs = msgs or MSGS
    results = {}
    # primer
    client(1, 1, 1024)
    time.sleep(1)
    # warm up
    client(1, max(concurrencies), 1024 * 100)
    time.sleep(2)
    # bench
    for concurrency in concurrencies:
        cres = results[concurrency] = {}
        for msg in msgs:
            res = client(10, concurrency, msg)
            cres[msg] = res
            time.sleep(3)
        time.sleep(1)
    return results


def raw():
    results = {}
    for loop in LOOPS:
        with server(loop):
            results[loop] = benchmark(concurrencies=[CONCURRENCIES[0]])
    return results


def stream():
    results = {}
    for loop in LOOPS:
        with server(loop, streams=True):
            results[loop] = benchmark(concurrencies=[CONCURRENCIES[0]])
    return results


def proto():
    results = {}
    for loop in LOOPS:
        with server(loop, proto=True):
            results[loop] = benchmark(concurrencies=[CONCURRENCIES[0]])
    return results


def concurrency():
    results = {}
    for loop in LOOPS:
        with server(loop):
            results[loop] = benchmark(msgs=[1024], concurrencies=CONCURRENCIES[1:])
    return results


def _rloop_version():
    import rloop

    return rloop.__version__


def run():
    all_benchmarks = {
        'raw': raw,
        'stream': stream,
        'proto': proto,
        'concurrency': concurrency,
    }
    inp_benchmarks = sys.argv[1:] or ['raw']
    run_benchmarks = set(inp_benchmarks) & set(all_benchmarks.keys())

    now = datetime.datetime.utcnow()
    results = {}
    for benchmark_key in run_benchmarks:
        runner = all_benchmarks[benchmark_key]
        results[benchmark_key] = runner()

    with open('results/data.json', 'w') as f:
        pyver = sys.version_info
        f.write(
            json.dumps(
                {
                    'cpu': CPU,
                    'run_at': int(now.timestamp()),
                    'pyver': f'{pyver.major}.{pyver.minor}',
                    'results': results,
                    'rloop': _rloop_version(),
                }
            )
        )


if __name__ == '__main__':
    run()
