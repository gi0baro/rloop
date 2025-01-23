# RLoop benchmarks

## Python versions

Run at: Thu 23 Jan 2025, 02:17    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.0a4    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 198081 | 19808.1 | 0.049ms | 0.068ms | 0.008 |
| 3.11 | 194015 | 19401.5 | 0.05ms | 0.068ms | 0.009 |
| 3.12 | 189599 | 18959.9 | 0.053ms | 0.072ms | 0.008 |
| 3.13 | 189258 | 18925.8 | 0.053ms | 0.072ms | 0.008 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 178638 | 17863.8 | 0.053ms | 0.074ms | 0.008 |
| 3.11 | 179001 | 17900.1 | 0.054ms | 0.074ms | 0.008 |
| 3.12 | 169237 | 16923.7 | 0.056ms | 0.077ms | 0.009 |
| 3.13 | 170217 | 17021.7 | 0.055ms | 0.077ms | 0.009 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 114593 | 11459.3 | 0.085ms | 0.116ms | 0.015 |
| 3.11 | 100321 | 10032.1 | 0.097ms | 0.141ms | 0.019 |
| 3.12 | 97906 | 9790.6 | 0.1ms | 0.142ms | 0.017 |
| 3.13 | 117274 | 11727.4 | 0.085ms | 0.108ms | 0.011 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 131240 | 13124.0 (73.5%) | 0.074ms | 0.106ms | 0.018 |
| 3.10 | rloop | 178638 | 17863.8 (100.0%) | 0.053ms | 0.074ms | 0.008 |
| 3.10 | uvloop | 149683 | 14968.3 (83.8%) | 0.065ms | 0.09ms | 0.011 |
| 3.11 | asyncio | 138602 | 13860.2 (77.4%) | 0.07ms | 0.099ms | 0.014 |
| 3.11 | rloop | 179001 | 17900.1 (100.0%) | 0.054ms | 0.074ms | 0.008 |
| 3.11 | uvloop | 151038 | 15103.8 (84.4%) | 0.065ms | 0.089ms | 0.011 |
| 3.12 | asyncio | 146853 | 14685.3 (86.8%) | 0.065ms | 0.101ms | 0.017 |
| 3.12 | rloop | 169237 | 16923.7 (100.0%) | 0.056ms | 0.077ms | 0.009 |
| 3.12 | uvloop | 143959 | 14395.9 (85.1%) | 0.066ms | 0.095ms | 0.012 |
| 3.13 | asyncio | 136782 | 13678.2 (80.4%) | 0.071ms | 0.096ms | 0.011 |
| 3.13 | rloop | 170217 | 17021.7 (100.0%) | 0.055ms | 0.077ms | 0.009 |
| 3.13 | uvloop | 142926 | 14292.6 (84.0%) | 0.067ms | 0.095ms | 0.013 |
