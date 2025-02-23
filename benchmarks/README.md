# RLoop benchmarks

Run at: Sun 23 Feb 2025, 18:38    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.0    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14766.9 (75.6%) | 14019.0 (78.3%) | 9900.4 (89.2%) | 
| rloop | 19544.8 (100.0%) | 17896.9 (100.0%) | 11103.9 (100.0%) | 
| uvloop | 16774.5 (85.8%) | 15214.8 (85.0%) | 9809.4 (88.3%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 147669 | 14766.9 (75.6%) | 0.066ms | 0.105ms | 0.015 |
| rloop | 195448 | 19544.8 (100.0%) | 0.051ms | 0.079ms | 0.007 |
| uvloop | 167745 | 16774.5 (85.8%) | 0.057ms | 0.097ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 140190 | 14019.0 (78.3%) | 0.069ms | 0.107ms | 0.013 |
| rloop | 178969 | 17896.9 (100.0%) | 0.054ms | 0.084ms | 0.008 |
| uvloop | 152148 | 15214.8 (85.0%) | 0.064ms | 0.1ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 99004 | 9900.4 (89.2%) | 0.098ms | 0.142ms | 0.013 |
| rloop | 111039 | 11103.9 (100.0%) | 0.089ms | 0.139ms | 0.015 |
| uvloop | 98094 | 9809.4 (88.3%) | 0.101ms | 0.18ms | 0.023 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14647.2 (86.2%) | 14095.1 (95.8%) | 5838.0 (86.0%) | 
| rloop | 16997.0 (100.0%) | 14719.7 (100.0%) | 6792.0 (100.0%) | 
| uvloop | 14942.0 (87.9%) | 13877.3 (94.3%) | 7234.0 (106.5%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 146472 | 14647.2 (86.2%) | 0.066ms | 0.099ms | 0.01 |
| rloop | 169970 | 16997.0 (100.0%) | 0.056ms | 0.085ms | 0.01 |
| uvloop | 149420 | 14942.0 (87.9%) | 0.066ms | 0.096ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 140951 | 14095.1 (95.8%) | 0.069ms | 0.1ms | 0.01 |
| rloop | 147197 | 14719.7 (100.0%) | 0.066ms | 0.095ms | 0.011 |
| uvloop | 138773 | 13877.3 (94.3%) | 0.068ms | 0.102ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 58380 | 5838.0 (86.0%) | 0.168ms | 0.239ms | 0.034 |
| rloop | 67920 | 6792.0 (100.0%) | 0.144ms | 0.204ms | 0.029 |
| uvloop | 72340 | 7234.0 (106.5%) | 0.136ms | 0.198ms | 0.025 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 18327.8 (84.4%) | 16887.7 (86.5%) | 11971.9 (96.3%) | 
| rloop | 21702.7 (100.0%) | 19526.7 (100.0%) | 12429.2 (100.0%) | 
| uvloop | 20257.7 (93.3%) | 17878.8 (91.6%) | 11769.1 (94.7%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 183278 | 18327.8 (84.4%) | 0.053ms | 0.08ms | 0.008 |
| rloop | 217027 | 21702.7 (100.0%) | 0.044ms | 0.068ms | 0.006 |
| uvloop | 202577 | 20257.7 (93.3%) | 0.047ms | 0.071ms | 0.009 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 168877 | 16887.7 (86.5%) | 0.057ms | 0.084ms | 0.01 |
| rloop | 195267 | 19526.7 (100.0%) | 0.05ms | 0.071ms | 0.008 |
| uvloop | 178788 | 17878.8 (91.6%) | 0.054ms | 0.08ms | 0.008 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 119719 | 11971.9 (96.3%) | 0.084ms | 0.116ms | 0.009 |
| rloop | 124292 | 12429.2 (100.0%) | 0.078ms | 0.109ms | 0.01 |
| uvloop | 117691 | 11769.1 (94.7%) | 0.083ms | 0.119ms | 0.008 |


### Other benchmarks

- [Python versions](./pyver.md)
