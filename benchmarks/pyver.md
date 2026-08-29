# RLoop benchmarks

## Python versions

Run at: Sat 29 Aug 2026, 23:16    
Environment: AMD Ryzen 7 5700X @ Gentoo Linux 6.18.43 (CPUs: 16)    
RLoop version: 0.4.1    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 457923 | 45792.3 | 0.021ms | 0.03ms | 0.004 |
| 3.11 | 463550 | 46355.0 | 0.021ms | 0.03ms | 0.004 |
| 3.12 | 469921 | 46992.1 | 0.021ms | 0.029ms | 0.004 |
| 3.13 | 474599 | 47459.9 | 0.02ms | 0.029ms | 0.005 |
| 3.14 | 464659 | 46465.9 | 0.021ms | 0.029ms | 0.004 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 405266 | 40526.6 | 0.022ms | 0.035ms | 0.004 |
| 3.11 | 413238 | 41323.8 | 0.022ms | 0.034ms | 0.004 |
| 3.12 | 407600 | 40760.0 | 0.022ms | 0.036ms | 0.008 |
| 3.13 | 406887 | 40688.7 | 0.023ms | 0.036ms | 0.006 |
| 3.14 | 410475 | 41047.5 | 0.022ms | 0.034ms | 0.007 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 130169 | 13016.9 | 0.076ms | 0.096ms | 0.02 |
| 3.11 | 128645 | 12864.5 | 0.077ms | 0.094ms | 0.022 |
| 3.12 | 138369 | 13836.9 | 0.071ms | 0.093ms | 0.02 |
| 3.13 | 130768 | 13076.8 | 0.075ms | 0.097ms | 0.021 |
| 3.14 | 122784 | 12278.4 | 0.08ms | 0.102ms | 0.024 |

