# RLoop benchmarks

## Python versions

Run at: Wed 17 Jun 2026, 13:47    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.3.0    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 253180 | 25318.0 | 0.037ms | 0.05ms | 0.007 |
| 3.11 | 254103 | 25410.3 | 0.037ms | 0.052ms | 0.007 |
| 3.12 | 251529 | 25152.9 | 0.038ms | 0.051ms | 0.007 |
| 3.13 | 250073 | 25007.3 | 0.039ms | 0.052ms | 0.007 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 227896 | 22789.6 | 0.042ms | 0.058ms | 0.007 |
| 3.11 | 222094 | 22209.4 | 0.043ms | 0.059ms | 0.007 |
| 3.12 | 218995 | 21899.5 | 0.044ms | 0.06ms | 0.007 |
| 3.13 | 229083 | 22908.3 | 0.042ms | 0.058ms | 0.008 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 100961 | 10096.1 | 0.097ms | 0.122ms | 0.017 |
| 3.11 | 101751 | 10175.1 | 0.096ms | 0.122ms | 0.018 |
| 3.12 | 100438 | 10043.8 | 0.097ms | 0.122ms | 0.018 |
| 3.13 | 102248 | 10224.8 | 0.096ms | 0.12ms | 0.017 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 202944 | 20294.4 (89.1%) | 0.047ms | 0.068ms | 0.01 |
| 3.10 | rloop | 227896 | 22789.6 (100.0%) | 0.042ms | 0.058ms | 0.007 |
| 3.10 | uvloop | 208950 | 20895.0 (91.7%) | 0.046ms | 0.062ms | 0.008 |
| 3.11 | asyncio | 196945 | 19694.5 (88.7%) | 0.048ms | 0.068ms | 0.01 |
| 3.11 | rloop | 222094 | 22209.4 (100.0%) | 0.043ms | 0.059ms | 0.007 |
| 3.11 | uvloop | 214747 | 21474.7 (96.7%) | 0.045ms | 0.06ms | 0.007 |
| 3.12 | asyncio | 215497 | 21549.7 (98.4%) | 0.045ms | 0.065ms | 0.009 |
| 3.12 | rloop | 218995 | 21899.5 (100.0%) | 0.044ms | 0.06ms | 0.007 |
| 3.12 | uvloop | 206193 | 20619.3 (94.2%) | 0.046ms | 0.062ms | 0.008 |
| 3.13 | asyncio | 189199 | 18919.9 (82.6%) | 0.051ms | 0.069ms | 0.01 |
| 3.13 | rloop | 229083 | 22908.3 (100.0%) | 0.042ms | 0.058ms | 0.008 |
| 3.13 | uvloop | 205524 | 20552.4 (89.7%) | 0.046ms | 0.064ms | 0.008 |
