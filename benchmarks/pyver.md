# RLoop benchmarks

## Python versions

Run at: Tue 12 Aug 2025, 13:07    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.5    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 173492 | 17349.2 | 0.056ms | 0.077ms | 0.01 |
| 3.11 | 174637 | 17463.7 | 0.056ms | 0.077ms | 0.01 |
| 3.12 | 168967 | 16896.7 | 0.057ms | 0.078ms | 0.01 |
| 3.13 | 173281 | 17328.1 | 0.056ms | 0.077ms | 0.01 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 150884 | 15088.4 | 0.064ms | 0.088ms | 0.012 |
| 3.11 | 149967 | 14996.7 | 0.064ms | 0.088ms | 0.013 |
| 3.12 | 144300 | 14430.0 | 0.067ms | 0.091ms | 0.013 |
| 3.13 | 159929 | 15992.9 | 0.059ms | 0.082ms | 0.011 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 74986 | 7498.6 | 0.131ms | 0.165ms | 0.031 |
| 3.11 | 74787 | 7478.7 | 0.131ms | 0.164ms | 0.03 |
| 3.12 | 73346 | 7334.6 | 0.133ms | 0.168ms | 0.031 |
| 3.13 | 71543 | 7154.3 | 0.137ms | 0.171ms | 0.031 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 136746 | 13674.6 (90.6%) | 0.072ms | 0.104ms | 0.015 |
| 3.10 | rloop | 150884 | 15088.4 (100.0%) | 0.064ms | 0.088ms | 0.012 |
| 3.10 | uvloop | 146153 | 14615.3 (96.9%) | 0.066ms | 0.095ms | 0.014 |
| 3.11 | asyncio | 135265 | 13526.5 (90.2%) | 0.072ms | 0.099ms | 0.014 |
| 3.11 | rloop | 149967 | 14996.7 (100.0%) | 0.064ms | 0.088ms | 0.013 |
| 3.11 | uvloop | 146669 | 14666.9 (97.8%) | 0.066ms | 0.095ms | 0.014 |
| 3.12 | asyncio | 146959 | 14695.9 (101.8%) | 0.065ms | 0.099ms | 0.016 |
| 3.12 | rloop | 144300 | 14430.0 (100.0%) | 0.067ms | 0.091ms | 0.013 |
| 3.12 | uvloop | 135345 | 13534.5 (93.8%) | 0.071ms | 0.1ms | 0.015 |
| 3.13 | asyncio | 130213 | 13021.3 (81.4%) | 0.073ms | 0.105ms | 0.017 |
| 3.13 | rloop | 159929 | 15992.9 (100.0%) | 0.059ms | 0.082ms | 0.011 |
| 3.13 | uvloop | 147904 | 14790.4 (92.5%) | 0.066ms | 0.096ms | 0.014 |
