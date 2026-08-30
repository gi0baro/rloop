# RLoop benchmarks

Run at: Sun 30 Aug 2026, 14:01    
Environment: AMD Ryzen 7 5700X @ Gentoo Linux 6.18.43 (CPUs: 16)    
Python version: 3.14    
RLoop version: 0.5.0    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 36844.9 (70.2%) | 32188.1 (76.5%) | 18876.7 (81.4%) | 
| rloop | 52502.6 (100.0%) | 42068.1 (100.0%) | 23188.7 (100.0%) | 
| uvloop | 45101.4 (85.9%) | 38301.5 (91.0%) | 15883.5 (68.5%) | 
| zuvloop | 50942.5 (97.0%) | 44994.0 (107.0%) | 20756.9 (89.5%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 368449 | 36844.9 (70.2%) | 0.026ms | 0.04ms | 0.005 |
| rloop | 525026 | 52502.6 (100.0%) | 0.02ms | 0.03ms | 0.001 |
| uvloop | 451014 | 45101.4 (85.9%) | 0.021ms | 0.039ms | 0.004 |
| zuvloop | 509425 | 50942.5 (97.0%) | 0.02ms | 0.03ms | 0.002 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 321881 | 32188.1 (76.5%) | 0.029ms | 0.046ms | 0.004 |
| rloop | 420681 | 42068.1 (100.0%) | 0.022ms | 0.039ms | 0.01 |
| uvloop | 383015 | 38301.5 (91.0%) | 0.024ms | 0.04ms | 0.006 |
| zuvloop | 449940 | 44994.0 (107.0%) | 0.02ms | 0.036ms | 0.003 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 188767 | 18876.7 (81.4%) | 0.051ms | 0.069ms | 0.004 |
| rloop | 231887 | 23188.7 (100.0%) | 0.041ms | 0.059ms | 0.004 |
| uvloop | 158835 | 15883.5 (68.5%) | 0.062ms | 0.092ms | 0.011 |
| zuvloop | 207569 | 20756.9 (89.5%) | 0.047ms | 0.063ms | 0.005 |


### Raw sockets (pipelined)

TCP echo server with raw sockets and a pipelined client (messages kept in flight, latency includes queueing) using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 320305.5 (119.0%) | 182514.9 (88.6%) | 35445.2 (97.8%) | 
| rloop | 269237.9 (100.0%) | 205992.1 (100.0%) | 36260.7 (100.0%) | 
| uvloop | 381120.3 (141.6%) | 193388.8 (93.9%) | 24244.0 (66.9%) | 
| zuvloop | 308784.4 (114.7%) | 206431.9 (100.2%) | 37712.8 (104.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 3203055 | 320305.5 (119.0%) | 0.035ms | 0.06ms | 0.106 |
| rloop | 2692379 | 269237.9 (100.0%) | 0.035ms | 0.064ms | 0.162 |
| uvloop | 3811203 | 381120.3 (141.6%) | 0.048ms | 0.072ms | 0.063 |
| zuvloop | 3087844 | 308784.4 (114.7%) | 0.033ms | 0.053ms | 0.092 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 1825149 | 182514.9 (88.6%) | 0.097ms | 1.517ms | 0.289 |
| rloop | 2059921 | 205992.1 (100.0%) | 0.095ms | 1.994ms | 0.337 |
| uvloop | 1933888 | 193388.8 (93.9%) | 0.181ms | 2.376ms | 0.423 |
| zuvloop | 2064319 | 206431.9 (100.2%) | 0.119ms | 2.318ms | 0.382 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 354452 | 35445.2 (97.8%) | 1.983ms | 5.632ms | 1.301 |
| rloop | 362607 | 36260.7 (100.0%) | 1.859ms | 5.401ms | 1.096 |
| uvloop | 242440 | 24244.0 (66.9%) | 1.329ms | 2.488ms | 0.378 |
| zuvloop | 377128 | 37712.8 (104.0%) | 1.02ms | 4.152ms | 0.709 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 40413.5 (82.5%) | 34349.4 (84.4%) | 17135.3 (92.0%) | 
| rloop | 48957.5 (100.0%) | 40715.4 (100.0%) | 18622.2 (100.0%) | 
| uvloop | 42491.3 (86.8%) | 36431.7 (89.5%) | 16807.8 (90.3%) | 
| zuvloop | 45255.1 (92.4%) | 37833.1 (92.9%) | 15720.4 (84.4%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 404135 | 40413.5 (82.5%) | 0.021ms | 0.039ms | 0.004 |
| rloop | 489575 | 48957.5 (100.0%) | 0.02ms | 0.03ms | 0.002 |
| uvloop | 424913 | 42491.3 (86.8%) | 0.021ms | 0.038ms | 0.003 |
| zuvloop | 452551 | 45255.1 (92.4%) | 0.021ms | 0.036ms | 0.003 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 343494 | 34349.4 (84.4%) | 0.03ms | 0.04ms | 0.002 |
| rloop | 407154 | 40715.4 (100.0%) | 0.022ms | 0.039ms | 0.004 |
| uvloop | 364317 | 36431.7 (89.5%) | 0.028ms | 0.04ms | 0.004 |
| zuvloop | 378331 | 37833.1 (92.9%) | 0.025ms | 0.04ms | 0.005 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 171353 | 17135.3 (92.0%) | 0.059ms | 0.074ms | 0.005 |
| rloop | 186222 | 18622.2 (100.0%) | 0.052ms | 0.069ms | 0.004 |
| uvloop | 168078 | 16807.8 (90.3%) | 0.06ms | 0.075ms | 0.004 |
| zuvloop | 157204 | 15720.4 (84.4%) | 0.061ms | 0.079ms | 0.004 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 54457.3 (94.5%) | 46998.2 (95.3%) | 22924.6 (95.0%) | 
| rloop | 57634.1 (100.0%) | 49338.4 (100.0%) | 24126.1 (100.0%) | 
| uvloop | 57872.1 (100.4%) | 49645.2 (100.6%) | 23184.6 (96.1%) | 
| zuvloop | 60146.7 (104.4%) | 50393.5 (102.1%) | 21619.6 (89.6%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 544573 | 54457.3 (94.5%) | 0.02ms | 0.03ms | 0.001 |
| rloop | 576341 | 57634.1 (100.0%) | 0.02ms | 0.03ms | 0.001 |
| uvloop | 578721 | 57872.1 (100.4%) | 0.02ms | 0.03ms | 0.001 |
| zuvloop | 601467 | 60146.7 (104.4%) | 0.02ms | 0.03ms | 0.0 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 469982 | 46998.2 (95.3%) | 0.02ms | 0.03ms | 0.001 |
| rloop | 493384 | 49338.4 (100.0%) | 0.02ms | 0.03ms | 0.001 |
| uvloop | 496452 | 49645.2 (100.6%) | 0.02ms | 0.03ms | 0.001 |
| zuvloop | 503935 | 50393.5 (102.1%) | 0.02ms | 0.03ms | 0.001 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 229246 | 22924.6 (95.0%) | 0.041ms | 0.058ms | 0.003 |
| rloop | 241261 | 24126.1 (100.0%) | 0.041ms | 0.057ms | 0.003 |
| uvloop | 231846 | 23184.6 (96.1%) | 0.04ms | 0.056ms | 0.003 |
| zuvloop | 216196 | 21619.6 (89.6%) | 0.044ms | 0.06ms | 0.005 |


### Other benchmarks

- [Python versions](./pyver.md)
