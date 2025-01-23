# RLoop benchmarks

Run at: Thu 23 Jan 2025, 02:16    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.0a4    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 15337.2 (78.7%) | 13439.7 (78.2%) | 9657.3 (87.0%) | 
| rloop | 19491.0 (100.0%) | 17192.6 (100.0%) | 11100.9 (100.0%) | 
| uvloop | 16293.7 (83.6%) | 15108.4 (87.9%) | 9442.7 (85.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 153372 | 15337.2 (78.7%) | 0.063ms | 0.102ms | 0.014 |
| rloop | 194910 | 19491.0 (100.0%) | 0.051ms | 0.08ms | 0.008 |
| uvloop | 162937 | 16293.7 (83.6%) | 0.059ms | 0.098ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 134397 | 13439.7 (78.2%) | 0.072ms | 0.112ms | 0.014 |
| rloop | 171926 | 17192.6 (100.0%) | 0.055ms | 0.088ms | 0.009 |
| uvloop | 151084 | 15108.4 (87.9%) | 0.064ms | 0.1ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 96573 | 9657.3 (87.0%) | 0.099ms | 0.146ms | 0.015 |
| rloop | 111009 | 11100.9 (100.0%) | 0.089ms | 0.144ms | 0.015 |
| uvloop | 94427 | 9442.7 (85.1%) | 0.104ms | 0.181ms | 0.022 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14497.7 (91.8%) | 13362.2 (88.8%) | 8995.0 (124.8%) | 
| rloop | 15789.5 (100.0%) | 15048.5 (100.0%) | 7208.3 (100.0%) | 
| uvloop | 14933.2 (94.6%) | 13842.7 (92.0%) | 7156.1 (99.3%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 144977 | 14497.7 (91.8%) | 0.067ms | 0.1ms | 0.011 |
| rloop | 157895 | 15789.5 (100.0%) | 0.06ms | 0.091ms | 0.011 |
| uvloop | 149332 | 14933.2 (94.6%) | 0.066ms | 0.096ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 133622 | 13362.2 (88.8%) | 0.072ms | 0.106ms | 0.012 |
| rloop | 150485 | 15048.5 (100.0%) | 0.065ms | 0.095ms | 0.011 |
| uvloop | 138427 | 13842.7 (92.0%) | 0.069ms | 0.103ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 89950 | 8995.0 (124.8%) | 0.108ms | 0.15ms | 0.136 |
| rloop | 72083 | 7208.3 (100.0%) | 0.137ms | 0.199ms | 0.155 |
| uvloop | 71561 | 7156.1 (99.3%) | 0.136ms | 0.196ms | 0.155 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 19221.2 (89.9%) | 17326.4 (91.3%) | 11904.1 (95.6%) | 
| rloop | 21385.6 (100.0%) | 18983.4 (100.0%) | 12449.8 (100.0%) | 
| uvloop | 20295.6 (94.9%) | 17917.5 (94.4%) | 12328.4 (99.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 192212 | 19221.2 (89.9%) | 0.052ms | 0.077ms | 0.007 |
| rloop | 213856 | 21385.6 (100.0%) | 0.044ms | 0.069ms | 0.007 |
| uvloop | 202956 | 20295.6 (94.9%) | 0.047ms | 0.072ms | 0.009 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 173264 | 17326.4 (91.3%) | 0.055ms | 0.082ms | 0.009 |
| rloop | 189834 | 18983.4 (100.0%) | 0.051ms | 0.075ms | 0.009 |
| uvloop | 179175 | 17917.5 (94.4%) | 0.054ms | 0.083ms | 0.009 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 119041 | 11904.1 (95.6%) | 0.083ms | 0.114ms | 0.008 |
| rloop | 124498 | 12449.8 (100.0%) | 0.076ms | 0.109ms | 0.01 |
| uvloop | 123284 | 12328.4 (99.0%) | 0.077ms | 0.111ms | 0.011 |


### Other benchmarks

- [Python versions](./pyver.md)
