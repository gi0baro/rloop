# RLoop benchmarks

Run at: Sat 20 Jun 2026, 16:23    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.3.1    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 12560.6 (80.5%) | 11235.3 (79.0%) | 8489.7 (120.5%) | 
| rloop | 15595.1 (100.0%) | 14218.9 (100.0%) | 7045.8 (100.0%) | 
| uvloop | 14204.6 (91.1%) | 13772.3 (96.9%) | 8856.2 (125.7%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 125606 | 12560.6 (80.5%) | 0.078ms | 0.119ms | 0.016 |
| rloop | 155951 | 15595.1 (100.0%) | 0.062ms | 0.097ms | 0.014 |
| uvloop | 142046 | 14204.6 (91.1%) | 0.068ms | 0.11ms | 0.015 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 112353 | 11235.3 (79.0%) | 0.086ms | 0.128ms | 0.019 |
| rloop | 142189 | 14218.9 (100.0%) | 0.068ms | 0.109ms | 0.016 |
| uvloop | 137723 | 13772.3 (96.9%) | 0.069ms | 0.113ms | 0.016 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 84897 | 8489.7 (120.5%) | 0.114ms | 0.146ms | 0.02 |
| rloop | 70458 | 7045.8 (100.0%) | 0.138ms | 0.287ms | 0.032 |
| uvloop | 88562 | 8856.2 (125.7%) | 0.11ms | 0.19ms | 0.023 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 13011.2 (86.4%) | 11956.3 (83.9%) | 5585.7 (91.7%) | 
| rloop | 15058.1 (100.0%) | 14242.5 (100.0%) | 6090.5 (100.0%) | 
| uvloop | 13587.9 (90.2%) | 12506.7 (87.8%) | 6451.0 (105.9%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 130112 | 13011.2 (86.4%) | 0.076ms | 0.111ms | 0.011 |
| rloop | 150581 | 15058.1 (100.0%) | 0.066ms | 0.097ms | 0.012 |
| uvloop | 135879 | 13587.9 (90.2%) | 0.07ms | 0.107ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 119563 | 11956.3 (83.9%) | 0.08ms | 0.118ms | 0.018 |
| rloop | 142425 | 14242.5 (100.0%) | 0.066ms | 0.102ms | 0.017 |
| uvloop | 125067 | 12506.7 (87.8%) | 0.077ms | 0.113ms | 0.012 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 55857 | 5585.7 (91.7%) | 0.175ms | 0.249ms | 0.038 |
| rloop | 60905 | 6090.5 (100.0%) | 0.161ms | 0.237ms | 0.038 |
| uvloop | 64510 | 6451.0 (105.9%) | 0.151ms | 0.218ms | 0.028 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 15804.4 (85.7%) | 14988.6 (88.4%) | 10586.0 (88.2%) | 
| rloop | 18446.5 (100.0%) | 16959.6 (100.0%) | 11998.4 (100.0%) | 
| uvloop | 17852.0 (96.8%) | 15891.9 (93.7%) | 10815.3 (90.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 158044 | 15804.4 (85.7%) | 0.059ms | 0.092ms | 0.011 |
| rloop | 184465 | 18446.5 (100.0%) | 0.053ms | 0.077ms | 0.009 |
| uvloop | 178520 | 17852.0 (96.8%) | 0.055ms | 0.083ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 149886 | 14988.6 (88.4%) | 0.065ms | 0.097ms | 0.01 |
| rloop | 169596 | 16959.6 (100.0%) | 0.056ms | 0.082ms | 0.012 |
| uvloop | 158919 | 15891.9 (93.7%) | 0.059ms | 0.09ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 105860 | 10586.0 (88.2%) | 0.09ms | 0.124ms | 0.023 |
| rloop | 119984 | 11998.4 (100.0%) | 0.079ms | 0.11ms | 0.013 |
| uvloop | 108153 | 10815.3 (90.1%) | 0.089ms | 0.118ms | 0.011 |


### Other benchmarks

- [Python versions](./pyver.md)
