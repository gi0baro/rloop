import argparse
import json
import math
import socket
import time
from concurrent import futures

import numpy as np


_FMT_OUT = """
{messages} {size}KiB messages in {duration} seconds
Latency: min {latency_min}ms; max {latency_max}ms; mean {latency_mean}ms;
std: {latency_std}ms ({latency_cv}%)
Latency distribution: {latency_percentiles}
Requests/sec: {rps}
Transfer/sec: {transfer}MiB"""


def wquant(values, quantiles, weights):
    values = np.array(values)
    quantiles = np.array(quantiles)
    weights = np.array(weights)
    assert np.all(quantiles >= 0) and np.all(quantiles <= 1), 'quantiles should be in [0, 1]'

    wqs = np.cumsum(weights) - 0.5 * weights
    wqs -= wqs[0]
    wqs /= wqs[-1]

    return np.interp(quantiles, wqs, values)


def bench(unix, addr, start, duration, timeout, reqsize, msg):
    if unix:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    else:
        sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)

    sock.settimeout(timeout / 1000)
    sock.connect(addr)

    n = 0
    latency_stats = np.zeros((timeout * 100,))
    min_latency = float('inf')
    max_latency = 0.0

    while time.monotonic() - start < duration:
        req_start = time.monotonic()
        sock.sendall(msg)
        nrecv = 0
        while nrecv < reqsize:
            resp = sock.recv(reqsize)
            if not resp:
                raise SystemExit()
            nrecv += len(resp)
        req_time = round((time.monotonic() - req_start) * 100000)
        if req_time > max_latency:
            max_latency = req_time
        if req_time < min_latency:
            min_latency = req_time
        latency_stats[req_time] += 1
        n += 1

    try:
        sock.close()
    except OSError:
        pass

    return n, latency_stats, min_latency, max_latency


def run(args):
    unix = False
    if args.addr.startswith('file:'):
        unix = True
        addr = args.addr[5:]
    else:
        addr = args.addr.split(':')
        addr[1] = int(addr[1])
        addr = tuple(addr)

    msg_size = args.msize
    msg = (b'x' * (msg_size - 1) + b'\n') * args.mpr

    req_size = msg_size * args.mpr

    timeout = args.timeout * 1000

    wrk = args.concurrency
    duration = args.duration

    min_latency = float('inf')
    max_latency = 0.0
    messages = 0
    latency_stats = None
    start = time.monotonic()

    with futures.ProcessPoolExecutor(max_workers=wrk) as e:
        fs = []
        for _ in range(wrk):
            fs.append(e.submit(bench, unix, addr, start, duration, timeout, req_size, msg))

        res = futures.wait(fs)
        for fut in res.done:
            t_messages, t_latency_stats, t_min_latency, t_max_latency = fut.result()
            messages += t_messages
            if latency_stats is None:
                latency_stats = t_latency_stats
            else:
                latency_stats = np.add(latency_stats, t_latency_stats)
            if t_max_latency > max_latency:
                max_latency = t_max_latency
            if t_min_latency < min_latency:
                min_latency = t_min_latency

    arange = np.arange(len(latency_stats))
    mean_latency = np.average(arange, weights=latency_stats)
    variance = np.average((arange - mean_latency) ** 2, weights=latency_stats)
    latency_std = math.sqrt(variance)
    latency_cv = latency_std / mean_latency

    percentiles = [50, 75, 90, 95, 99]
    percentile_data = []

    quantiles = wquant(arange, [p / 100 for p in percentiles], weights=latency_stats)

    for i, percentile in enumerate(percentiles):
        percentile_data.append((percentile, round(quantiles[i] / 100, 3)))

    data = {
        'messages': messages,
        'transfer': round((messages * msg_size / (1024 * 1024)) / duration, 2),
        'rps': round(messages / duration, 2),
        'latency_min': round(min_latency / 100, 3),
        'latency_mean': round(mean_latency / 100, 3),
        'latency_max': round(max_latency / 100, 3),
        'latency_std': round(latency_std / 100, 3),
        'latency_cv': round(latency_cv * 100, 2),
        'latency_percentiles': percentile_data,
    }

    if args.output == 'json':
        print(json.dumps(data))
    else:
        data['latency_percentiles'] = '; '.join('{}% under {}ms'.format(*v) for v in percentile_data)
        output = _FMT_OUT.format(duration=duration, size=round(msg_size / 1024, 2), **data)
        print(output)


if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument('--msize', default=1024, type=int, help='message size in bytes')
    parser.add_argument('--mpr', default=1, type=int, help='messages per request')
    parser.add_argument('--duration', '-T', default=10, type=int, help='duration of test in seconds')
    parser.add_argument('--concurrency', default=1, type=int, help='request concurrency')
    parser.add_argument('--timeout', default=2, type=int, help='socket timeout in seconds')
    parser.add_argument('--addr', default='127.0.0.1:25000', type=str, help='server address')
    parser.add_argument('--output', default='text', type=str, help='output format', choices=['text', 'json'])
    run(parser.parse_args())
