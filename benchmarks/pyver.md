# RLoop benchmarks

## Python versions

Run at: Thu 30 Jan 2025, 02:28    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.0a5    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 192587 | 19258.7 | 0.048ms | 0.069ms | 0.01 |
| 3.11 | 198926 | 19892.6 | 0.046ms | 0.068ms | 0.01 |
| 3.12 | 192247 | 19224.7 | 0.052ms | 0.072ms | 0.008 |
| 3.13 | 198171 | 19817.1 | 0.049ms | 0.066ms | 0.008 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 181245 | 18124.5 | 0.054ms | 0.074ms | 0.008 |
| 3.11 | 183810 | 18381.0 | 0.053ms | 0.074ms | 0.008 |
| 3.12 | 175385 | 17538.5 | 0.054ms | 0.075ms | 0.008 |
| 3.13 | 179275 | 17927.5 | 0.053ms | 0.074ms | 0.008 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 110252 | 11025.2 | 0.088ms | 0.121ms | 0.015 |
| 3.11 | 104607 | 10460.7 | 0.093ms | 0.13ms | 0.018 |
| 3.12 | 116811 | 11681.1 | 0.085ms | 0.109ms | 0.011 |
| 3.13 | 110247 | 11024.7 | 0.089ms | 0.127ms | 0.018 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 149026 | 14902.6 (82.2%) | 0.067ms | 0.095ms | 0.012 |
| 3.10 | rloop | 181245 | 18124.5 (100.0%) | 0.054ms | 0.074ms | 0.008 |
| 3.10 | uvloop | 149781 | 14978.1 (82.6%) | 0.065ms | 0.089ms | 0.011 |
| 3.11 | asyncio | 148548 | 14854.8 (80.8%) | 0.066ms | 0.095ms | 0.013 |
| 3.11 | rloop | 183810 | 18381.0 (100.0%) | 0.053ms | 0.074ms | 0.008 |
| 3.11 | uvloop | 150073 | 15007.3 (81.6%) | 0.065ms | 0.09ms | 0.012 |
| 3.12 | asyncio | 154792 | 15479.2 (88.3%) | 0.062ms | 0.094ms | 0.014 |
| 3.12 | rloop | 175385 | 17538.5 (100.0%) | 0.054ms | 0.075ms | 0.008 |
| 3.12 | uvloop | 139965 | 13996.5 (79.8%) | 0.068ms | 0.097ms | 0.014 |
| 3.13 | asyncio | 131151 | 13115.1 (73.2%) | 0.073ms | 0.099ms | 0.015 |
| 3.13 | rloop | 179275 | 17927.5 (100.0%) | 0.053ms | 0.074ms | 0.008 |
| 3.13 | uvloop | 149585 | 14958.5 (83.4%) | 0.065ms | 0.092ms | 0.011 |
