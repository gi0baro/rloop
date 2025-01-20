# RLoop benchmarks

Run at: Mon 20 Jan 2025, 22:39    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.0a4    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 15173.5 (81.5%) | 13528.3 (78.0%) | 9499.4 (83.9%) | 
| rloop | 18624.9 (100.0%) | 17342.3 (100.0%) | 11321.4 (100.0%) | 
| uvloop | 16570.5 (89.0%) | 14628.4 (84.4%) | 9856.6 (87.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 151735 | 15173.5 (81.5%) | 0.064ms | 0.104ms | 0.024 |
| rloop | 186249 | 18624.9 (100.0%) | 0.053ms | 0.08ms | 0.008 |
| uvloop | 165705 | 16570.5 (89.0%) | 0.058ms | 0.096ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 135283 | 13528.3 (78.0%) | 0.072ms | 0.113ms | 0.018 |
| rloop | 173423 | 17342.3 (100.0%) | 0.054ms | 0.086ms | 0.008 |
| uvloop | 146284 | 14628.4 (84.4%) | 0.066ms | 0.104ms | 0.013 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 94994 | 9499.4 (83.9%) | 0.101ms | 0.148ms | 0.029 |
| rloop | 113214 | 11321.4 (100.0%) | 0.086ms | 0.129ms | 0.012 |
| uvloop | 98566 | 9856.6 (87.1%) | 0.101ms | 0.18ms | 0.022 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14543.8 (92.8%) | 12985.7 (96.0%) | 9048.5 (117.2%) | 
| rloop | 15665.9 (100.0%) | 13532.0 (100.0%) | 7717.4 (100.0%) | 
| uvloop | 15337.1 (97.9%) | 13530.8 (100.0%) | 7180.7 (93.0%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 145438 | 14543.8 (92.8%) | 0.066ms | 0.1ms | 0.011 |
| rloop | 156659 | 15665.9 (100.0%) | 0.064ms | 0.091ms | 0.01 |
| uvloop | 153371 | 15337.1 (97.9%) | 0.064ms | 0.093ms | 0.015 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 129857 | 12985.7 (96.0%) | 0.075ms | 0.108ms | 0.011 |
| rloop | 135320 | 13532.0 (100.0%) | 0.07ms | 0.104ms | 0.013 |
| uvloop | 135308 | 13530.8 (100.0%) | 0.07ms | 0.103ms | 0.013 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 90485 | 9048.5 (117.2%) | 0.106ms | 0.146ms | 0.135 |
| rloop | 77174 | 7717.4 (100.0%) | 0.126ms | 0.177ms | 0.149 |
| uvloop | 71807 | 7180.7 (93.0%) | 0.137ms | 0.201ms | 0.154 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 18246.7 (100.1%) | 16834.7 (100.4%) | 11670.4 (105.0%) | 
| rloop | 18221.3 (100.0%) | 16770.4 (100.0%) | 11113.2 (100.0%) | 
| uvloop | 19844.9 (108.9%) | 18308.3 (109.2%) | 12273.5 (110.4%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 182467 | 18246.7 (100.1%) | 0.054ms | 0.079ms | 0.007 |
| rloop | 182213 | 18221.3 (100.0%) | 0.054ms | 0.078ms | 0.007 |
| uvloop | 198449 | 19844.9 (108.9%) | 0.048ms | 0.072ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 168347 | 16834.7 (100.4%) | 0.057ms | 0.083ms | 0.009 |
| rloop | 167704 | 16770.4 (100.0%) | 0.057ms | 0.082ms | 0.01 |
| uvloop | 183083 | 18308.3 (109.2%) | 0.053ms | 0.079ms | 0.007 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 116704 | 11670.4 (105.0%) | 0.085ms | 0.12ms | 0.01 |
| rloop | 111132 | 11113.2 (100.0%) | 0.086ms | 0.12ms | 0.01 |
| uvloop | 122735 | 12273.5 (110.4%) | 0.078ms | 0.11ms | 0.01 |


### Other benchmarks

- [Python versions](./pyver.md)
