# RLoop benchmarks

## Python versions

Run at: Thu 17 Apr 2025, 17:18    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.1    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 204732 | 20473.2 | 0.045ms | 0.065ms | 0.008 |
| 3.11 | 196732 | 19673.2 | 0.048ms | 0.067ms | 0.01 |
| 3.12 | 188072 | 18807.2 | 0.052ms | 0.069ms | 0.008 |
| 3.13 | 189527 | 18952.7 | 0.053ms | 0.073ms | 0.009 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 177102 | 17710.2 | 0.055ms | 0.076ms | 0.012 |
| 3.11 | 182910 | 18291.0 | 0.053ms | 0.073ms | 0.007 |
| 3.12 | 174548 | 17454.8 | 0.055ms | 0.076ms | 0.009 |
| 3.13 | 172471 | 17247.1 | 0.055ms | 0.076ms | 0.009 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 115367 | 11536.7 | 0.084ms | 0.122ms | 0.016 |
| 3.11 | 114159 | 11415.9 | 0.087ms | 0.124ms | 0.015 |
| 3.12 | 116232 | 11623.2 | 0.085ms | 0.11ms | 0.012 |
| 3.13 | 94847 | 9484.7 | 0.104ms | 0.142ms | 0.021 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 148582 | 14858.2 (83.9%) | 0.067ms | 0.097ms | 0.013 |
| 3.10 | rloop | 177102 | 17710.2 (100.0%) | 0.055ms | 0.076ms | 0.012 |
| 3.10 | uvloop | 144995 | 14499.5 (81.9%) | 0.067ms | 0.092ms | 0.013 |
| 3.11 | asyncio | 147801 | 14780.1 (80.8%) | 0.066ms | 0.096ms | 0.014 |
| 3.11 | rloop | 182910 | 18291.0 (100.0%) | 0.053ms | 0.073ms | 0.007 |
| 3.11 | uvloop | 150928 | 15092.8 (82.5%) | 0.065ms | 0.088ms | 0.011 |
| 3.12 | asyncio | 159524 | 15952.4 (91.4%) | 0.059ms | 0.093ms | 0.014 |
| 3.12 | rloop | 174548 | 17454.8 (100.0%) | 0.055ms | 0.076ms | 0.009 |
| 3.12 | uvloop | 144773 | 14477.3 (82.9%) | 0.066ms | 0.093ms | 0.012 |
| 3.13 | asyncio | 127934 | 12793.4 (74.2%) | 0.076ms | 0.104ms | 0.015 |
| 3.13 | rloop | 172471 | 17247.1 (100.0%) | 0.055ms | 0.076ms | 0.009 |
| 3.13 | uvloop | 147221 | 14722.1 (85.4%) | 0.065ms | 0.093ms | 0.012 |
