# RLoop benchmarks

## Python versions

Run at: Sun 23 Feb 2025, 18:38    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.0    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 194417 | 19441.7 | 0.05ms | 0.068ms | 0.009 |
| 3.11 | 198728 | 19872.8 | 0.048ms | 0.067ms | 0.009 |
| 3.12 | 184606 | 18460.6 | 0.054ms | 0.074ms | 0.008 |
| 3.13 | 194319 | 19431.9 | 0.052ms | 0.069ms | 0.007 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 178385 | 17838.5 | 0.054ms | 0.075ms | 0.008 |
| 3.11 | 179994 | 17999.4 | 0.054ms | 0.076ms | 0.009 |
| 3.12 | 170939 | 17093.9 | 0.055ms | 0.077ms | 0.009 |
| 3.13 | 176808 | 17680.8 | 0.054ms | 0.075ms | 0.008 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 109561 | 10956.1 | 0.09ms | 0.125ms | 0.016 |
| 3.11 | 111221 | 11122.1 | 0.087ms | 0.12ms | 0.015 |
| 3.12 | 115370 | 11537.0 | 0.086ms | 0.109ms | 0.011 |
| 3.13 | 112606 | 11260.6 | 0.088ms | 0.12ms | 0.015 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 151912 | 15191.2 (85.2%) | 0.066ms | 0.093ms | 0.011 |
| 3.10 | rloop | 178385 | 17838.5 (100.0%) | 0.054ms | 0.075ms | 0.008 |
| 3.10 | uvloop | 151489 | 15148.9 (84.9%) | 0.065ms | 0.088ms | 0.011 |
| 3.11 | asyncio | 155875 | 15587.5 (86.6%) | 0.063ms | 0.091ms | 0.012 |
| 3.11 | rloop | 179994 | 17999.4 (100.0%) | 0.054ms | 0.076ms | 0.009 |
| 3.11 | uvloop | 153787 | 15378.7 (85.4%) | 0.064ms | 0.087ms | 0.01 |
| 3.12 | asyncio | 152613 | 15261.3 (89.3%) | 0.063ms | 0.097ms | 0.016 |
| 3.12 | rloop | 170939 | 17093.9 (100.0%) | 0.055ms | 0.077ms | 0.009 |
| 3.12 | uvloop | 142951 | 14295.1 (83.6%) | 0.067ms | 0.095ms | 0.012 |
| 3.13 | asyncio | 133649 | 13364.9 (75.6%) | 0.072ms | 0.099ms | 0.015 |
| 3.13 | rloop | 176808 | 17680.8 (100.0%) | 0.054ms | 0.075ms | 0.008 |
| 3.13 | uvloop | 155701 | 15570.1 (88.1%) | 0.063ms | 0.089ms | 0.011 |
