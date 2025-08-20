# RLoop benchmarks

## Python versions

Run at: Wed 20 Aug 2025, 17:49    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.6    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 166367 | 16636.7 | 0.057ms | 0.078ms | 0.012 |
| 3.11 | 168212 | 16821.2 | 0.057ms | 0.078ms | 0.01 |
| 3.12 | 160149 | 16014.9 | 0.058ms | 0.08ms | 0.011 |
| 3.13 | 164049 | 16404.9 | 0.057ms | 0.079ms | 0.012 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 143889 | 14388.9 | 0.067ms | 0.09ms | 0.013 |
| 3.11 | 145148 | 14514.8 | 0.066ms | 0.089ms | 0.012 |
| 3.12 | 137552 | 13755.2 | 0.07ms | 0.096ms | 0.013 |
| 3.13 | 151036 | 15103.6 | 0.063ms | 0.088ms | 0.012 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 74686 | 7468.6 | 0.131ms | 0.164ms | 0.031 |
| 3.11 | 74573 | 7457.3 | 0.131ms | 0.171ms | 0.033 |
| 3.12 | 72518 | 7251.8 | 0.134ms | 0.171ms | 0.032 |
| 3.13 | 75040 | 7504.0 | 0.13ms | 0.167ms | 0.033 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 142436 | 14243.6 (99.0%) | 0.069ms | 0.101ms | 0.015 |
| 3.10 | rloop | 143889 | 14388.9 (100.0%) | 0.067ms | 0.09ms | 0.013 |
| 3.10 | uvloop | 143446 | 14344.6 (99.7%) | 0.067ms | 0.096ms | 0.015 |
| 3.11 | asyncio | 132500 | 13250.0 (91.3%) | 0.073ms | 0.1ms | 0.016 |
| 3.11 | rloop | 145148 | 14514.8 (100.0%) | 0.066ms | 0.089ms | 0.012 |
| 3.11 | uvloop | 139344 | 13934.4 (96.0%) | 0.069ms | 0.097ms | 0.015 |
| 3.12 | asyncio | 137740 | 13774.0 (100.1%) | 0.07ms | 0.105ms | 0.018 |
| 3.12 | rloop | 137552 | 13755.2 (100.0%) | 0.07ms | 0.096ms | 0.013 |
| 3.12 | uvloop | 136740 | 13674.0 (99.4%) | 0.071ms | 0.102ms | 0.014 |
| 3.13 | asyncio | 124749 | 12474.9 (82.6%) | 0.077ms | 0.108ms | 0.016 |
| 3.13 | rloop | 151036 | 15103.6 (100.0%) | 0.063ms | 0.088ms | 0.012 |
| 3.13 | uvloop | 129997 | 12999.7 (86.1%) | 0.074ms | 0.106ms | 0.016 |
