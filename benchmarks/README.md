# RLoop benchmarks

Run at: Sat 29 Aug 2026, 23:04    
Environment: AMD Ryzen 7 5700X @ Gentoo Linux 6.18.43 (CPUs: 16)    
Python version: 3.14    
RLoop version: 0.4.1    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 34958.0 (76.5%) | 31330.6 (76.3%) | 18103.9 (132.9%) | 
| rloop | 45671.8 (100.0%) | 41041.9 (100.0%) | 13622.1 (100.0%) | 
| uvloop | 42532.7 (93.1%) | 35103.0 (85.5%) | 18054.0 (132.5%) | 
| zuvloop | 50498.2 (110.6%) | 44557.0 (108.6%) | 19622.9 (144.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 349580 | 34958.0 (76.5%) | 0.03ms | 0.044ms | 0.004 |
| rloop | 456718 | 45671.8 (100.0%) | 0.021ms | 0.037ms | 0.006 |
| uvloop | 425327 | 42532.7 (93.1%) | 0.024ms | 0.04ms | 0.007 |
| zuvloop | 504982 | 50498.2 (110.6%) | 0.021ms | 0.037ms | 0.005 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 313306 | 31330.6 (76.3%) | 0.031ms | 0.047ms | 0.003 |
| rloop | 410419 | 41041.9 (100.0%) | 0.022ms | 0.039ms | 0.009 |
| uvloop | 351030 | 35103.0 (85.5%) | 0.029ms | 0.044ms | 0.006 |
| zuvloop | 445570 | 44557.0 (108.6%) | 0.021ms | 0.039ms | 0.004 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 181039 | 18103.9 (132.9%) | 0.054ms | 0.072ms | 0.008 |
| rloop | 136221 | 13622.1 (100.0%) | 0.071ms | 0.153ms | 0.019 |
| uvloop | 180540 | 18054.0 (132.5%) | 0.055ms | 0.079ms | 0.009 |
| zuvloop | 196229 | 19622.9 (144.1%) | 0.049ms | 0.069ms | 0.011 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 39961.4 (84.1%) | 34879.1 (87.0%) | 13591.8 (94.0%) | 
| rloop | 47521.9 (100.0%) | 40094.8 (100.0%) | 14456.0 (100.0%) | 
| uvloop | 41690.0 (87.7%) | 35853.1 (89.4%) | 17604.7 (121.8%) | 
| zuvloop | 43957.9 (92.5%) | 37138.3 (92.6%) | 17922.7 (124.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 399614 | 39961.4 (84.1%) | 0.022ms | 0.039ms | 0.005 |
| rloop | 475219 | 47521.9 (100.0%) | 0.02ms | 0.03ms | 0.004 |
| uvloop | 416900 | 41690.0 (87.7%) | 0.021ms | 0.039ms | 0.006 |
| zuvloop | 439579 | 43957.9 (92.5%) | 0.021ms | 0.039ms | 0.004 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 348791 | 34879.1 (87.0%) | 0.029ms | 0.042ms | 0.006 |
| rloop | 400948 | 40094.8 (100.0%) | 0.022ms | 0.039ms | 0.005 |
| uvloop | 358531 | 35853.1 (89.4%) | 0.028ms | 0.04ms | 0.005 |
| zuvloop | 371383 | 37138.3 (92.6%) | 0.026ms | 0.04ms | 0.006 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 135918 | 13591.8 (94.0%) | 0.073ms | 0.108ms | 0.018 |
| rloop | 144560 | 14456.0 (100.0%) | 0.067ms | 0.102ms | 0.016 |
| uvloop | 176047 | 17604.7 (121.8%) | 0.055ms | 0.07ms | 0.009 |
| zuvloop | 179227 | 17922.7 (124.0%) | 0.054ms | 0.073ms | 0.007 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 53441.9 (94.9%) | 45376.4 (91.3%) | 23704.3 (91.4%) | 
| rloop | 56322.8 (100.0%) | 49702.8 (100.0%) | 25948.6 (100.0%) | 
| uvloop | 55829.5 (99.1%) | 47734.4 (96.0%) | 21644.2 (83.4%) | 
| zuvloop | 56695.7 (100.7%) | 49050.5 (98.7%) | 24914.5 (96.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 534419 | 53441.9 (94.9%) | 0.02ms | 0.03ms | 0.002 |
| rloop | 563228 | 56322.8 (100.0%) | 0.02ms | 0.03ms | 0.001 |
| uvloop | 558295 | 55829.5 (99.1%) | 0.02ms | 0.03ms | 0.001 |
| zuvloop | 566957 | 56695.7 (100.7%) | 0.02ms | 0.03ms | 0.002 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 453764 | 45376.4 (91.3%) | 0.021ms | 0.037ms | 0.003 |
| rloop | 497028 | 49702.8 (100.0%) | 0.02ms | 0.03ms | 0.001 |
| uvloop | 477344 | 47734.4 (96.0%) | 0.02ms | 0.03ms | 0.004 |
| zuvloop | 490505 | 49050.5 (98.7%) | 0.02ms | 0.03ms | 0.002 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 237043 | 23704.3 (91.4%) | 0.042ms | 0.06ms | 0.009 |
| rloop | 259486 | 25948.6 (100.0%) | 0.039ms | 0.053ms | 0.005 |
| uvloop | 216442 | 21644.2 (83.4%) | 0.043ms | 0.06ms | 0.006 |
| zuvloop | 249145 | 24914.5 (96.0%) | 0.04ms | 0.054ms | 0.003 |


### Other benchmarks

- [Python versions](./pyver.md)
