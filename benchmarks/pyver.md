# RLoop benchmarks

## Python versions

Run at: Sat 20 Jun 2026, 16:24    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.3.1    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 142866 | 14286.6 | 0.067ms | 0.089ms | 0.011 |
| 3.11 | 146599 | 14659.9 | 0.066ms | 0.088ms | 0.011 |
| 3.12 | 142661 | 14266.1 | 0.067ms | 0.091ms | 0.012 |
| 3.13 | 140601 | 14060.1 | 0.068ms | 0.092ms | 0.012 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 137154 | 13715.4 | 0.07ms | 0.097ms | 0.013 |
| 3.11 | 131195 | 13119.5 | 0.073ms | 0.099ms | 0.015 |
| 3.12 | 131165 | 13116.5 | 0.073ms | 0.1ms | 0.016 |
| 3.13 | 135346 | 13534.6 | 0.071ms | 0.096ms | 0.012 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 69671 | 6967.1 | 0.14ms | 0.181ms | 0.034 |
| 3.11 | 69504 | 6950.4 | 0.141ms | 0.18ms | 0.031 |
| 3.12 | 65801 | 6580.1 | 0.148ms | 0.186ms | 0.033 |
| 3.13 | 66096 | 6609.6 | 0.147ms | 0.189ms | 0.035 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 103452 | 10345.2 (75.4%) | 0.093ms | 0.127ms | 0.019 |
| 3.10 | rloop | 137154 | 13715.4 (100.0%) | 0.07ms | 0.097ms | 0.013 |
| 3.10 | uvloop | 128206 | 12820.6 (93.5%) | 0.074ms | 0.106ms | 0.015 |
| 3.11 | asyncio | 117030 | 11703.0 (89.2%) | 0.082ms | 0.11ms | 0.015 |
| 3.11 | rloop | 131195 | 13119.5 (100.0%) | 0.073ms | 0.099ms | 0.015 |
| 3.11 | uvloop | 128752 | 12875.2 (98.1%) | 0.075ms | 0.107ms | 0.015 |
| 3.12 | asyncio | 109856 | 10985.6 (83.8%) | 0.089ms | 0.118ms | 0.017 |
| 3.12 | rloop | 131165 | 13116.5 (100.0%) | 0.073ms | 0.1ms | 0.016 |
| 3.12 | uvloop | 121661 | 12166.1 (92.8%) | 0.079ms | 0.113ms | 0.019 |
| 3.13 | asyncio | 113312 | 11331.2 (83.7%) | 0.084ms | 0.119ms | 0.018 |
| 3.13 | rloop | 135346 | 13534.6 (100.0%) | 0.071ms | 0.096ms | 0.012 |
| 3.13 | uvloop | 124606 | 12460.6 (92.1%) | 0.076ms | 0.112ms | 0.019 |
