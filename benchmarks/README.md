# RLoop benchmarks

Run at: Thu 30 Jan 2025, 02:26    
Environment: GHA Linux x86_64 (CPUs: 4)    
Python version: 3.13    
RLoop version: 0.1.0a5    

### Raw sockets

TCP echo server with raw sockets comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14939.9 (76.4%) | 13355.8 (74.7%) | 9582.8 (84.9%) | 
| rloop | 19545.4 (100.0%) | 17878.1 (100.0%) | 11281.3 (100.0%) | 
| uvloop | 16288.9 (83.3%) | 14681.7 (82.1%) | 9937.1 (88.1%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 149399 | 14939.9 (76.4%) | 0.065ms | 0.104ms | 0.018 |
| rloop | 195454 | 19545.4 (100.0%) | 0.05ms | 0.08ms | 0.01 |
| uvloop | 162889 | 16288.9 (83.3%) | 0.059ms | 0.099ms | 0.013 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 133558 | 13355.8 (74.7%) | 0.072ms | 0.112ms | 0.015 |
| rloop | 178781 | 17878.1 (100.0%) | 0.054ms | 0.085ms | 0.011 |
| uvloop | 146817 | 14681.7 (82.1%) | 0.066ms | 0.106ms | 0.014 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 95828 | 9582.8 (84.9%) | 0.1ms | 0.146ms | 0.015 |
| rloop | 112813 | 11281.3 (100.0%) | 0.086ms | 0.138ms | 0.016 |
| uvloop | 99371 | 9937.1 (88.1%) | 0.1ms | 0.179ms | 0.022 |


### Streams

TCP echo server with `asyncio` streams comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 14260.8 (84.1%) | 13196.9 (86.0%) | 6065.4 (81.4%) | 
| rloop | 16964.3 (100.0%) | 15342.9 (100.0%) | 7454.6 (100.0%) | 
| uvloop | 15013.2 (88.5%) | 14103.9 (91.9%) | 7192.7 (96.5%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 142608 | 14260.8 (84.1%) | 0.068ms | 0.104ms | 0.013 |
| rloop | 169643 | 16964.3 (100.0%) | 0.057ms | 0.088ms | 0.012 |
| uvloop | 150132 | 15013.2 (88.5%) | 0.065ms | 0.096ms | 0.012 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 131969 | 13196.9 (86.0%) | 0.073ms | 0.108ms | 0.015 |
| rloop | 153429 | 15342.9 (100.0%) | 0.064ms | 0.094ms | 0.011 |
| uvloop | 141039 | 14103.9 (91.9%) | 0.067ms | 0.101ms | 0.011 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 60654 | 6065.4 (81.4%) | 0.162ms | 0.229ms | 0.038 |
| rloop | 74546 | 7454.6 (100.0%) | 0.132ms | 0.189ms | 0.025 |
| uvloop | 71927 | 7192.7 (96.5%) | 0.137ms | 0.201ms | 0.026 |


### Protocol

TCP echo server with `asyncio.Protocol` comparison using 1KB, 10KB and 100KB messages.


| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
| asyncio | 19004.8 (90.8%) | 17040.5 (88.1%) | 12155.0 (97.8%) | 
| rloop | 20925.4 (100.0%) | 19350.4 (100.0%) | 12431.8 (100.0%) | 
| uvloop | 19758.3 (94.4%) | 18294.1 (94.5%) | 12462.8 (100.2%) | 


#### 1KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 190048 | 19004.8 (90.8%) | 0.052ms | 0.079ms | 0.009 |
| rloop | 209254 | 20925.4 (100.0%) | 0.045ms | 0.069ms | 0.01 |
| uvloop | 197583 | 19758.3 (94.4%) | 0.048ms | 0.074ms | 0.01 |


#### 10KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 170405 | 17040.5 (88.1%) | 0.057ms | 0.086ms | 0.011 |
| rloop | 193504 | 19350.4 (100.0%) | 0.05ms | 0.073ms | 0.008 |
| uvloop | 182941 | 18294.1 (94.5%) | 0.053ms | 0.079ms | 0.009 |


#### 100KB details

| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
| asyncio | 121550 | 12155.0 (97.8%) | 0.081ms | 0.113ms | 0.011 |
| rloop | 124318 | 12431.8 (100.0%) | 0.076ms | 0.116ms | 0.018 |
| uvloop | 124628 | 12462.8 (100.2%) | 0.076ms | 0.11ms | 0.016 |


### Other benchmarks

- [Python versions](./pyver.md)
