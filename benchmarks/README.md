# RLoop benchmarks

Run at: Mon 26 May 2025, 18:42    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.2    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14362.1 (79.5%) | 14279.9 (90.7%) | 10020.0 (109.8%) | 
| rloop | 18059.6 (100.0%) | 15735.7 (100.0%) | 9128.7 (100.0%) | 
| uvloop | 15982.0 (88.5%) | 15114.6 (96.1%) | 10431.2 (114.3%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 143621 | 14362.1 (79.5%) | 0.067ms | 0.104ms | 0.012 |
| rloop | 180596 | 18059.6 (100.0%) | 0.054ms | 0.083ms | 0.01 |
| uvloop | 159820 | 15982.0 (88.5%) | 0.06ms | 0.097ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 142799 | 14279.9 (90.7%) | 0.067ms | 0.108ms | 0.013 |
| rloop | 157357 | 15735.7 (100.0%) | 0.061ms | 0.098ms | 0.012 |
| uvloop | 151146 | 15114.6 (96.1%) | 0.065ms | 0.101ms | 0.014 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 100200 | 10020.0 (109.8%) | 0.098ms | 0.142ms | 0.013 |
| rloop | 91287 | 9128.7 (100.0%) | 0.108ms | 0.229ms | 0.03 |
| uvloop | 104312 | 10431.2 (114.3%) | 0.093ms | 0.163ms | 0.018 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14640.4 (88.8%) | 13088.7 (84.3%) | 6129.7 (82.7%) | 
| rloop | 16480.6 (100.0%) | 15521.0 (100.0%) | 7408.6 (100.0%) | 
| uvloop | 15448.0 (93.7%) | 13617.3 (87.7%) | 7096.9 (95.8%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 146404 | 14640.4 (88.8%) | 0.067ms | 0.098ms | 0.01 |
| rloop | 164806 | 16480.6 (100.0%) | 0.059ms | 0.087ms | 0.01 |
| uvloop | 154480 | 15448.0 (93.7%) | 0.062ms | 0.091ms | 0.011 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 130887 | 13088.7 (84.3%) | 0.073ms | 0.106ms | 0.013 |
| rloop | 155210 | 15521.0 (100.0%) | 0.062ms | 0.091ms | 0.011 |
| uvloop | 136173 | 13617.3 (87.7%) | 0.07ms | 0.101ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 61297 | 6129.7 (82.7%) | 0.16ms | 0.228ms | 0.035 |
| rloop | 74086 | 7408.6 (100.0%) | 0.132ms | 0.189ms | 0.025 |
| uvloop | 70969 | 7096.9 (95.8%) | 0.137ms | 0.199ms | 0.025 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 17969.2 (82.9%) | 17152.6 (89.5%) | 12183.6 (102.3%) | 
| rloop | 21680.6 (100.0%) | 19159.4 (100.0%) | 11904.4 (100.0%) | 
| uvloop | 20472.7 (94.4%) | 18104.6 (94.5%) | 12347.8 (103.7%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 179692 | 17969.2 (82.9%) | 0.054ms | 0.08ms | 0.01 |
| rloop | 216806 | 21680.6 (100.0%) | 0.044ms | 0.068ms | 0.006 |
| uvloop | 204727 | 20472.7 (94.4%) | 0.046ms | 0.07ms | 0.011 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 171526 | 17152.6 (89.5%) | 0.056ms | 0.081ms | 0.009 |
| rloop | 191594 | 19159.4 (100.0%) | 0.051ms | 0.07ms | 0.008 |
| uvloop | 181046 | 18104.6 (94.5%) | 0.054ms | 0.079ms | 0.007 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 121836 | 12183.6 (102.3%) | 0.079ms | 0.113ms | 0.011 |
| rloop | 119044 | 11904.4 (100.0%) | 0.082ms | 0.109ms | 0.007 |
| uvloop | 123478 | 12347.8 (103.7%) | 0.077ms | 0.11ms | 0.01 |


### Other benchmarks

- [Python versions](./pyver.md)
