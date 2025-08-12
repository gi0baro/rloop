# RLoop benchmarks

Run at: Tue 12 Aug 2025, 13:06    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.5    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14139.9 (83.8%) | 13542.7 (84.7%) | 9885.4 (137.3%) | 
| rloop | 16865.0 (100.0%) | 15981.5 (100.0%) | 7200.1 (100.0%) | 
| uvloop | 15950.0 (94.6%) | 14635.9 (91.6%) | 9886.7 (137.3%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 141399 | 14139.9 (83.8%) | 0.068ms | 0.111ms | 0.016 |
| rloop | 168650 | 16865.0 (100.0%) | 0.057ms | 0.089ms | 0.012 |
| uvloop | 159500 | 15950.0 (94.6%) | 0.06ms | 0.1ms | 0.014 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 135427 | 13542.7 (84.7%) | 0.071ms | 0.114ms | 0.015 |
| rloop | 159815 | 15981.5 (100.0%) | 0.059ms | 0.093ms | 0.011 |
| uvloop | 146359 | 14635.9 (91.6%) | 0.067ms | 0.108ms | 0.014 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 98854 | 9885.4 (137.3%) | 0.099ms | 0.144ms | 0.014 |
| rloop | 72001 | 7200.1 (100.0%) | 0.136ms | 0.294ms | 0.033 |
| uvloop | 98867 | 9886.7 (137.3%) | 0.097ms | 0.159ms | 0.019 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14303.6 (89.5%) | 12526.1 (85.3%) | 5851.4 (80.3%) | 
| rloop | 15988.5 (100.0%) | 14691.7 (100.0%) | 7286.5 (100.0%) | 
| uvloop | 15342.7 (96.0%) | 13402.4 (91.2%) | 7307.8 (100.3%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 143036 | 14303.6 (89.5%) | 0.067ms | 0.1ms | 0.011 |
| rloop | 159885 | 15988.5 (100.0%) | 0.059ms | 0.091ms | 0.011 |
| uvloop | 153427 | 15342.7 (96.0%) | 0.065ms | 0.094ms | 0.009 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 125261 | 12526.1 (85.3%) | 0.077ms | 0.109ms | 0.013 |
| rloop | 146917 | 14691.7 (100.0%) | 0.066ms | 0.095ms | 0.01 |
| uvloop | 134024 | 13402.4 (91.2%) | 0.07ms | 0.103ms | 0.012 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 58514 | 5851.4 (80.3%) | 0.167ms | 0.239ms | 0.037 |
| rloop | 72865 | 7286.5 (100.0%) | 0.135ms | 0.202ms | 0.028 |
| uvloop | 73078 | 7307.8 (100.3%) | 0.135ms | 0.199ms | 0.025 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 18289.1 (88.3%) | 15803.4 (82.3%) | 11687.4 (94.6%) | 
| rloop | 20709.3 (100.0%) | 19197.7 (100.0%) | 12359.0 (100.0%) | 
| uvloop | 20394.7 (98.5%) | 18134.7 (94.5%) | 12183.2 (98.6%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 182891 | 18289.1 (88.3%) | 0.053ms | 0.079ms | 0.007 |
| rloop | 207093 | 20709.3 (100.0%) | 0.045ms | 0.069ms | 0.007 |
| uvloop | 203947 | 20394.7 (98.5%) | 0.046ms | 0.07ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 158034 | 15803.4 (82.3%) | 0.06ms | 0.089ms | 0.011 |
| rloop | 191977 | 19197.7 (100.0%) | 0.05ms | 0.071ms | 0.008 |
| uvloop | 181347 | 18134.7 (94.5%) | 0.054ms | 0.079ms | 0.007 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 116874 | 11687.4 (94.6%) | 0.085ms | 0.12ms | 0.01 |
| rloop | 123590 | 12359.0 (100.0%) | 0.077ms | 0.109ms | 0.01 |
| uvloop | 121832 | 12183.2 (98.6%) | 0.077ms | 0.114ms | 0.012 |


### Other benchmarks

- [Python versions](./pyver.md)
