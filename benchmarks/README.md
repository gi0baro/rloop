# RLoop benchmarks

Run at: Mon 13 Jan 2025, 19:52    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.12    
RLoop version: 0.1.0a3    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 17320.9 (91.1%) | 15300.1 (89.4%) | 9311.7 (93.9%) | 
| rloop | 19012.4 (100.0%) | 17106.7 (100.0%) | 9912.4 (100.0%) | 
| uvloop | 16432.1 (86.4%) | 13590.9 (79.4%) | 9382.9 (94.7%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 173209 | 17320.9 (91.1%) | 0.057ms | 0.099ms | 0.012 |
| rloop | 190124 | 19012.4 (100.0%) | 0.052ms | 0.082ms | 0.009 |
| uvloop | 164321 | 16432.1 (86.4%) | 0.059ms | 0.099ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 153001 | 15300.1 (89.4%) | 0.062ms | 0.109ms | 0.016 |
| rloop | 171067 | 17106.7 (100.0%) | 0.056ms | 0.088ms | 0.01 |
| uvloop | 135909 | 13590.9 (79.4%) | 0.07ms | 0.112ms | 0.016 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 93117 | 9311.7 (93.9%) | 0.104ms | 0.156ms | 0.016 |
| rloop | 99124 | 9912.4 (100.0%) | 0.098ms | 0.162ms | 0.023 |
| uvloop | 93829 | 9382.9 (94.7%) | 0.105ms | 0.171ms | 0.019 |


### Other benchmarks

- [Python versions](./pyver.md)
