# RLoop benchmarks

## Python versions

Run at: Sun 30 Aug 2026, 14:17    
Environment: AMD Ryzen 7 5700X @ Gentoo Linux 6.18.43 (CPUs: 16)    
RLoop version: 0.5.0    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 511594 | 51159.4 | 0.02ms | 0.029ms | 0.005 |
| 3.11 | 518008 | 51800.8 | 0.02ms | 0.029ms | 0.002 |
| 3.12 | 520033 | 52003.3 | 0.02ms | 0.029ms | 0.002 |
| 3.13 | 525737 | 52573.7 | 0.02ms | 0.029ms | 0.001 |
| 3.14 | 524838 | 52483.8 | 0.02ms | 0.029ms | 0.001 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 439379 | 43937.9 | 0.021ms | 0.03ms | 0.003 |
| 3.11 | 446528 | 44652.8 | 0.02ms | 0.029ms | 0.003 |
| 3.12 | 449498 | 44949.8 | 0.02ms | 0.029ms | 0.002 |
| 3.13 | 453553 | 45355.3 | 0.02ms | 0.029ms | 0.002 |
| 3.14 | 450071 | 45007.1 | 0.02ms | 0.029ms | 0.003 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 216302 | 21630.2 | 0.044ms | 0.058ms | 0.005 |
| 3.11 | 223446 | 22344.6 | 0.042ms | 0.054ms | 0.004 |
| 3.12 | 216745 | 21674.5 | 0.045ms | 0.058ms | 0.007 |
| 3.13 | 226073 | 22607.3 | 0.041ms | 0.053ms | 0.004 |
| 3.14 | 233909 | 23390.9 | 0.041ms | 0.05ms | 0.003 |

