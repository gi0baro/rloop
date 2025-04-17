# RLoop benchmarks

Run at: Thu 17 Apr 2025, 17:17    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.1    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 15729.6 (81.6%) | 13852.6 (79.0%) | 9692.1 (83.2%) | 
| rloop | 19272.9 (100.0%) | 17539.4 (100.0%) | 11652.4 (100.0%) | 
| uvloop | 16181.9 (84.0%) | 14564.8 (83.0%) | 9903.4 (85.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 157296 | 15729.6 (81.6%) | 0.061ms | 0.101ms | 0.014 |
| rloop | 192729 | 19272.9 (100.0%) | 0.052ms | 0.08ms | 0.008 |
| uvloop | 161819 | 16181.9 (84.0%) | 0.06ms | 0.099ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 138526 | 13852.6 (79.0%) | 0.07ms | 0.109ms | 0.013 |
| rloop | 175394 | 17539.4 (100.0%) | 0.054ms | 0.086ms | 0.009 |
| uvloop | 145648 | 14564.8 (83.0%) | 0.066ms | 0.103ms | 0.012 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 96921 | 9692.1 (83.2%) | 0.099ms | 0.145ms | 0.015 |
| rloop | 116524 | 11652.4 (100.0%) | 0.085ms | 0.126ms | 0.011 |
| uvloop | 99034 | 9903.4 (85.0%) | 0.1ms | 0.177ms | 0.023 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14531.2 (87.3%) | 13677.9 (90.5%) | 5972.8 (82.4%) | 
| rloop | 16643.7 (100.0%) | 15116.7 (100.0%) | 7251.9 (100.0%) | 
| uvloop | 15230.3 (91.5%) | 13067.2 (86.4%) | 7185.7 (99.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 145312 | 14531.2 (87.3%) | 0.066ms | 0.099ms | 0.01 |
| rloop | 166437 | 16643.7 (100.0%) | 0.057ms | 0.087ms | 0.01 |
| uvloop | 152303 | 15230.3 (91.5%) | 0.065ms | 0.093ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 136779 | 13677.9 (90.5%) | 0.071ms | 0.102ms | 0.011 |
| rloop | 151167 | 15116.7 (100.0%) | 0.065ms | 0.093ms | 0.01 |
| uvloop | 130672 | 13067.2 (86.4%) | 0.072ms | 0.105ms | 0.013 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 59728 | 5972.8 (82.4%) | 0.164ms | 0.23ms | 0.034 |
| rloop | 72519 | 7251.9 (100.0%) | 0.136ms | 0.197ms | 0.025 |
| uvloop | 71857 | 7185.7 (99.1%) | 0.137ms | 0.198ms | 0.025 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 19217.5 (90.3%) | 16426.5 (84.3%) | 12014.6 (94.9%) | 
| rloop | 21274.4 (100.0%) | 19495.2 (100.0%) | 12659.6 (100.0%) | 
| uvloop | 20171.2 (94.8%) | 17920.6 (91.9%) | 12295.2 (97.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 192175 | 19217.5 (90.3%) | 0.052ms | 0.077ms | 0.006 |
| rloop | 212744 | 21274.4 (100.0%) | 0.044ms | 0.068ms | 0.006 |
| uvloop | 201712 | 20171.2 (94.8%) | 0.047ms | 0.07ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 164265 | 16426.5 (84.3%) | 0.059ms | 0.086ms | 0.011 |
| rloop | 194952 | 19495.2 (100.0%) | 0.05ms | 0.07ms | 0.008 |
| uvloop | 179206 | 17920.6 (91.9%) | 0.054ms | 0.079ms | 0.007 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 120146 | 12014.6 (94.9%) | 0.083ms | 0.114ms | 0.008 |
| rloop | 126596 | 12659.6 (100.0%) | 0.075ms | 0.107ms | 0.009 |
| uvloop | 122952 | 12295.2 (97.1%) | 0.077ms | 0.112ms | 0.011 |


### Other benchmarks

- [Python versions](./pyver.md)
