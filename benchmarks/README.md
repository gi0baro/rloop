# RLoop benchmarks

Run at: Wed 17 Jun 2026, 13:46    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.3.0    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 12965.5 (88.9%) | 12003.9 (87.0%) | 8422.1 (117.0%) | 
| rloop | 14587.3 (100.0%) | 13792.4 (100.0%) | 7200.1 (100.0%) | 
| uvloop | 13669.7 (93.7%) | 12373.4 (89.7%) | 9116.8 (126.6%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 129655 | 12965.5 (88.9%) | 0.076ms | 0.114ms | 0.013 |
| rloop | 145873 | 14587.3 (100.0%) | 0.065ms | 0.099ms | 0.012 |
| uvloop | 136697 | 13669.7 (93.7%) | 0.07ms | 0.11ms | 0.018 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 120039 | 12003.9 (87.0%) | 0.08ms | 0.127ms | 0.018 |
| rloop | 137924 | 13792.4 (100.0%) | 0.07ms | 0.109ms | 0.013 |
| uvloop | 123734 | 12373.4 (89.7%) | 0.077ms | 0.119ms | 0.019 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 84221 | 8422.1 (117.0%) | 0.116ms | 0.151ms | 0.02 |
| rloop | 72001 | 7200.1 (100.0%) | 0.135ms | 0.268ms | 0.036 |
| uvloop | 91168 | 9116.8 (126.6%) | 0.106ms | 0.153ms | 0.02 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 11919.7 (88.6%) | 10894.9 (83.1%) | 7489.6 (113.3%) | 
| rloop | 13446.9 (100.0%) | 13109.9 (100.0%) | 6611.8 (100.0%) | 
| uvloop | 12555.7 (93.4%) | 11669.4 (89.0%) | 6518.7 (98.6%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 119197 | 11919.7 (88.6%) | 0.08ms | 0.11ms | 0.014 |
| rloop | 134469 | 13446.9 (100.0%) | 0.07ms | 0.1ms | 0.013 |
| uvloop | 125557 | 12555.7 (93.4%) | 0.076ms | 0.107ms | 0.011 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 108949 | 10894.9 (83.1%) | 0.089ms | 0.12ms | 0.013 |
| rloop | 131099 | 13109.9 (100.0%) | 0.074ms | 0.105ms | 0.01 |
| uvloop | 116694 | 11669.4 (89.0%) | 0.082ms | 0.111ms | 0.013 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 74896 | 7489.6 (113.3%) | 0.129ms | 0.165ms | 0.019 |
| rloop | 66118 | 6611.8 (100.0%) | 0.149ms | 0.216ms | 0.027 |
| uvloop | 65187 | 6518.7 (98.6%) | 0.15ms | 0.217ms | 0.027 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14832.5 (86.2%) | 13257.1 (83.9%) | 10077.2 (99.0%) | 
| rloop | 17203.8 (100.0%) | 15799.2 (100.0%) | 10177.2 (100.0%) | 
| uvloop | 16483.9 (95.8%) | 15163.6 (96.0%) | 10373.0 (101.9%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 148325 | 14832.5 (86.2%) | 0.066ms | 0.09ms | 0.01 |
| rloop | 172038 | 17203.8 (100.0%) | 0.057ms | 0.08ms | 0.01 |
| uvloop | 164839 | 16483.9 (95.8%) | 0.056ms | 0.083ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 132571 | 13257.1 (83.9%) | 0.071ms | 0.099ms | 0.013 |
| rloop | 157992 | 15799.2 (100.0%) | 0.06ms | 0.088ms | 0.01 |
| uvloop | 151636 | 15163.6 (96.0%) | 0.064ms | 0.09ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 100772 | 10077.2 (99.0%) | 0.096ms | 0.129ms | 0.013 |
| rloop | 101772 | 10177.2 (100.0%) | 0.095ms | 0.126ms | 0.014 |
| uvloop | 103730 | 10373.0 (101.9%) | 0.092ms | 0.12ms | 0.015 |


### Other benchmarks

- [Python versions](./pyver.md)
