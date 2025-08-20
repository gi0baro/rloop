# RLoop benchmarks

Run at: Wed 20 Aug 2025, 17:47    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.6    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14410.8 (87.5%) | 13549.6 (89.1%) | 9882.8 (116.5%) | 
| rloop | 16472.2 (100.0%) | 15214.9 (100.0%) | 8485.9 (100.0%) | 
| uvloop | 15889.1 (96.5%) | 14276.3 (93.8%) | 9376.6 (110.5%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 144108 | 14410.8 (87.5%) | 0.068ms | 0.105ms | 0.012 |
| rloop | 164722 | 16472.2 (100.0%) | 0.058ms | 0.089ms | 0.011 |
| uvloop | 158891 | 15889.1 (96.5%) | 0.06ms | 0.1ms | 0.014 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 135496 | 13549.6 (89.1%) | 0.071ms | 0.113ms | 0.02 |
| rloop | 152149 | 15214.9 (100.0%) | 0.063ms | 0.1ms | 0.014 |
| uvloop | 142763 | 14276.3 (93.8%) | 0.068ms | 0.109ms | 0.014 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 98828 | 9882.8 (116.5%) | 0.1ms | 0.143ms | 0.014 |
| rloop | 84859 | 8485.9 (100.0%) | 0.115ms | 0.24ms | 0.032 |
| uvloop | 93766 | 9376.6 (110.5%) | 0.103ms | 0.166ms | 0.019 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14196.4 (86.6%) | 12849.9 (87.1%) | 5915.2 (82.6%) | 
| rloop | 16401.5 (100.0%) | 14751.1 (100.0%) | 7161.5 (100.0%) | 
| uvloop | 14735.5 (89.8%) | 13427.1 (91.0%) | 6186.9 (86.4%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 141964 | 14196.4 (86.6%) | 0.067ms | 0.101ms | 0.011 |
| rloop | 164015 | 16401.5 (100.0%) | 0.057ms | 0.089ms | 0.01 |
| uvloop | 147355 | 14735.5 (89.8%) | 0.067ms | 0.097ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 128499 | 12849.9 (87.1%) | 0.075ms | 0.108ms | 0.012 |
| rloop | 147511 | 14751.1 (100.0%) | 0.066ms | 0.096ms | 0.01 |
| uvloop | 134271 | 13427.1 (91.0%) | 0.072ms | 0.105ms | 0.012 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 59152 | 5915.2 (82.6%) | 0.166ms | 0.236ms | 0.035 |
| rloop | 71615 | 7161.5 (100.0%) | 0.136ms | 0.197ms | 0.027 |
| uvloop | 61869 | 6186.9 (86.4%) | 0.159ms | 0.237ms | 0.038 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 17784.4 (85.3%) | 16494.6 (85.8%) | 11408.8 (96.0%) | 
| rloop | 20838.8 (100.0%) | 19225.1 (100.0%) | 11881.8 (100.0%) | 
| uvloop | 20296.3 (97.4%) | 18002.2 (93.6%) | 8027.6 (67.6%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 177844 | 17784.4 (85.3%) | 0.054ms | 0.081ms | 0.007 |
| rloop | 208388 | 20838.8 (100.0%) | 0.045ms | 0.069ms | 0.007 |
| uvloop | 202963 | 20296.3 (97.4%) | 0.046ms | 0.07ms | 0.009 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 164946 | 16494.6 (85.8%) | 0.057ms | 0.088ms | 0.01 |
| rloop | 192251 | 19225.1 (100.0%) | 0.05ms | 0.072ms | 0.008 |
| uvloop | 180022 | 18002.2 (93.6%) | 0.054ms | 0.08ms | 0.007 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 114088 | 11408.8 (96.0%) | 0.085ms | 0.121ms | 0.01 |
| rloop | 118818 | 11881.8 (100.0%) | 0.081ms | 0.112ms | 0.009 |
| uvloop | 80276 | 8027.6 (67.6%) | 0.122ms | 0.157ms | 0.01 |


### Other benchmarks

- [Python versions](./pyver.md)
