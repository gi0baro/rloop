# RLoop benchmarks

## Python versions

Run at: Mon 13 Jan 2025, 19:53    
Environment: GHA Linux x86_64 (CPUs: 4)    
RLoop version: 0.1.0a3    

Comparison between different Python versions.    
The only test performed is the raw socket one.


### 1KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 187746 | 18774.6 | 0.051ms | 0.069ms | 0.009 |
| 3.11 | 200452 | 20045.2 | 0.048ms | 0.067ms | 0.009 |
| 3.12 | 195912 | 19591.2 | 0.05ms | 0.07ms | 0.008 |
| 3.13 | 198162 | 19816.2 | 0.049ms | 0.068ms | 0.009 |


### 10KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 178376 | 17837.6 | 0.054ms | 0.075ms | 0.008 |
| 3.11 | 185560 | 18556.0 | 0.052ms | 0.072ms | 0.008 |
| 3.12 | 175623 | 17562.3 | 0.054ms | 0.075ms | 0.008 |
| 3.13 | 178564 | 17856.4 | 0.054ms | 0.075ms | 0.009 |


### 100KB

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| 3.10 | 101496 | 10149.6 | 0.095ms | 0.139ms | 0.02 |
| 3.11 | 113393 | 11339.3 | 0.086ms | 0.12ms | 0.015 |
| 3.12 | 102600 | 10260.0 | 0.095ms | 0.127ms | 0.014 |
| 3.13 | 110312 | 11031.2 | 0.089ms | 0.129ms | 0.017 |


### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
| 3.10 | asyncio | 140773 | 14077.3 (78.9%) | 0.07ms | 0.1ms | 0.016 |
| 3.10 | rloop | 178376 | 17837.6 (100.0%) | 0.054ms | 0.075ms | 0.008 |
| 3.10 | uvloop | 152942 | 15294.2 (85.7%) | 0.064ms | 0.087ms | 0.01 |
| 3.11 | asyncio | 147811 | 14781.1 (79.7%) | 0.066ms | 0.092ms | 0.011 |
| 3.11 | rloop | 185560 | 18556.0 (100.0%) | 0.052ms | 0.072ms | 0.008 |
| 3.11 | uvloop | 155553 | 15555.3 (83.8%) | 0.062ms | 0.087ms | 0.011 |
| 3.12 | asyncio | 154762 | 15476.2 (88.1%) | 0.062ms | 0.092ms | 0.016 |
| 3.12 | rloop | 175623 | 17562.3 (100.0%) | 0.054ms | 0.075ms | 0.008 |
| 3.12 | uvloop | 145771 | 14577.1 (83.0%) | 0.066ms | 0.094ms | 0.013 |
| 3.13 | asyncio | 140190 | 14019.0 (78.5%) | 0.069ms | 0.095ms | 0.012 |
| 3.13 | rloop | 178564 | 17856.4 (100.0%) | 0.054ms | 0.075ms | 0.009 |
| 3.13 | uvloop | 146504 | 14650.4 (82.0%) | 0.065ms | 0.092ms | 0.011 |
