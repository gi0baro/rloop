# RLoop benchmarks

## Python versions

Run at: Mon 20 Jan 2025, 22:39    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.0a4    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 195804 | 19580.4 | 0.048ms | 0.069ms | 0.01 |
| 3.11 | 194003 | 19400.3 | 0.051ms | 0.069ms | 0.008 |
| 3.12 | 194586 | 19458.6 | 0.052ms | 0.071ms | 0.007 |
| 3.13 | 190319 | 19031.9 | 0.053ms | 0.074ms | 0.009 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 176090 | 17609.0 | 0.054ms | 0.076ms | 0.009 |
| 3.11 | 175267 | 17526.7 | 0.054ms | 0.076ms | 0.009 |
| 3.12 | 173480 | 17348.0 | 0.055ms | 0.077ms | 0.009 |
| 3.13 | 172741 | 17274.1 | 0.055ms | 0.076ms | 0.009 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 111838 | 11183.8 | 0.088ms | 0.115ms | 0.014 |
| 3.11 | 111397 | 11139.7 | 0.088ms | 0.112ms | 0.012 |
| 3.12 | 112524 | 11252.4 | 0.087ms | 0.11ms | 0.012 |
| 3.13 | 111562 | 11156.2 | 0.088ms | 0.11ms | 0.012 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 145937 | 14593.7 (82.9%) | 0.068ms | 0.096ms | 0.013 |
| 3.10 | rloop | 176090 | 17609.0 (100.0%) | 0.054ms | 0.076ms | 0.009 |
| 3.10 | uvloop | 156391 | 15639.1 (88.8%) | 0.063ms | 0.086ms | 0.01 |
| 3.11 | asyncio | 143871 | 14387.1 (82.1%) | 0.067ms | 0.096ms | 0.013 |
| 3.11 | rloop | 175267 | 17526.7 (100.0%) | 0.054ms | 0.076ms | 0.009 |
| 3.11 | uvloop | 148750 | 14875.0 (84.9%) | 0.066ms | 0.089ms | 0.012 |
| 3.12 | asyncio | 157590 | 15759.0 (90.8%) | 0.06ms | 0.095ms | 0.014 |
| 3.12 | rloop | 173480 | 17348.0 (100.0%) | 0.055ms | 0.077ms | 0.009 |
| 3.12 | uvloop | 148090 | 14809.0 (85.4%) | 0.065ms | 0.093ms | 0.012 |
| 3.13 | asyncio | 134050 | 13405.0 (77.6%) | 0.072ms | 0.099ms | 0.013 |
| 3.13 | rloop | 172741 | 17274.1 (100.0%) | 0.055ms | 0.076ms | 0.009 |
| 3.13 | uvloop | 145135 | 14513.5 (84.0%) | 0.067ms | 0.094ms | 0.013 |
