# RLoop benchmarks

## Python versions

{{ _common_data = globals().get(f"data{pyvb}") }}
Run at: {{ =datetime.datetime.fromtimestamp(_common_data.run_at).strftime('%a %d %b %Y, %H:%M') }}    
Environment: {{ =benv }} (CPUs: {{ =_common_data.cpu }})    
RLoop version: {{ =_common_data.rloop }}    

Comparison between different Python versions.    
The only test performed is the raw socket one.

{{ for mkey, label in [("1024", "1KB"), ("10240", "10KB"), ("102400", "100KB")]: }}

### {{ =label }}

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
{{ for pykey in ["310", "311", "312", "313"]: }}
{{ _data = globals().get(f"data{pykey}") }}
{{ if not _data: }}
{{ continue }}
{{ bdata = _data.results["raw"]["rloop"]["1"][mkey] }}
| {{ =_data.pyver }} | {{ =bdata["messages"] }} | {{ =bdata["rps"] }} | {{ =f"{bdata['latency_mean']}ms" }} | {{ =f"{bdata['latency_percentiles'][-2][1]}ms" }} | {{ =bdata["latency_std"] }} |
{{ pass }}

{{ pass }}

### 10KB VS other

| Python version | Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- | --- |
{{ for pykey in ["310", "311", "312", "313"]: }}
{{ _data = globals().get(f"data{pykey}") }}
{{ if not _data: }}
{{ continue }}
{{ dcmp = _data.results["raw"]["rloop"]["1"]["10240"]["rps"] }}
{{ for lkey, bdata in _data.results["raw"].items(): }}
{{ lbdata = bdata["1"]["10240"] }}
| {{ =_data.pyver }} | {{ =lkey }} | {{ =lbdata["messages"] }} | {{ =lbdata["rps"] }} ({{ =round(lbdata["rps"] / dcmp * 100, 1) }}%) | {{ =f"{lbdata['latency_mean']}ms" }} | {{ =f"{lbdata['latency_percentiles'][-2][1]}ms" }} | {{ =lbdata["latency_std"] }} |
{{ pass }}
{{ pass }}
