# RLoop benchmarks

## Python versions

Run at: Mon 26 May 2025, 18:43    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.2    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 183734 | 18373.4 | 0.053ms | 0.073ms | 0.009 |
| 3.11 | 182561 | 18256.1 | 0.053ms | 0.075ms | 0.01 |
| 3.12 | 179827 | 17982.7 | 0.054ms | 0.077ms | 0.011 |
| 3.13 | 181028 | 18102.8 | 0.054ms | 0.076ms | 0.01 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 158534 | 15853.4 | 0.06ms | 0.084ms | 0.012 |
| 3.11 | 158605 | 15860.5 | 0.06ms | 0.086ms | 0.012 |
| 3.12 | 149465 | 14946.5 | 0.064ms | 0.089ms | 0.014 |
| 3.13 | 164537 | 16453.7 | 0.058ms | 0.079ms | 0.011 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 74532 | 7453.2 | 0.132ms | 0.171ms | 0.031 |
| 3.11 | 74820 | 7482.0 | 0.131ms | 0.164ms | 0.035 |
| 3.12 | 76570 | 7657.0 | 0.128ms | 0.166ms | 0.034 |
| 3.13 | 74302 | 7430.2 | 0.132ms | 0.165ms | 0.03 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 153639 | 15363.9 (96.9%) | 0.065ms | 0.093ms | 0.012 |
| 3.10 | rloop | 158534 | 15853.4 (100.0%) | 0.06ms | 0.084ms | 0.012 |
| 3.10 | uvloop | 156487 | 15648.7 (98.7%) | 0.061ms | 0.089ms | 0.014 |
| 3.11 | asyncio | 145181 | 14518.1 (91.5%) | 0.067ms | 0.097ms | 0.019 |
| 3.11 | rloop | 158605 | 15860.5 (100.0%) | 0.06ms | 0.086ms | 0.012 |
| 3.11 | uvloop | 154231 | 15423.1 (97.2%) | 0.062ms | 0.089ms | 0.014 |
| 3.12 | asyncio | 159232 | 15923.2 (106.5%) | 0.06ms | 0.091ms | 0.014 |
| 3.12 | rloop | 149465 | 14946.5 (100.0%) | 0.064ms | 0.089ms | 0.014 |
| 3.12 | uvloop | 151907 | 15190.7 (101.6%) | 0.064ms | 0.095ms | 0.014 |
| 3.13 | asyncio | 138677 | 13867.7 (84.3%) | 0.07ms | 0.098ms | 0.013 |
| 3.13 | rloop | 164537 | 16453.7 (100.0%) | 0.058ms | 0.079ms | 0.011 |
| 3.13 | uvloop | 155618 | 15561.8 (94.6%) | 0.063ms | 0.093ms | 0.015 |
