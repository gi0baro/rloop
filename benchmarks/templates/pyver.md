# RLoop benchmarks

## Python versions

{{ _common_data = globals().get(f"data{pyvb}") }}
Run at: {{ =datetime.datetime.fromtimestamp(_common_data.run_at).strftime('%a %d %b %Y, %H:%M') }}    
Environment: {{ =benv }} (CPUs: {{ =_common_data.cpu }})    
RLoop version: {{ =_common_data.rloop }}    

Comparison between different Python versions.    
The only test performed is the raw socket one with a message size of 1KB.

| Python version | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
{{ for pykey in ["310", "311", "312", "313"]: }}
{{ _data = globals().get(f"data{pykey}") }}
{{ if not _data: }}
{{ continue }}
{{ bdata = _data.results["raw"]["rloop"]["1"]["1024"] }}
{{ max_c, run = get_max_concurrency_run(runs) }}
| {{ =_data.pyver }} | {{ =bdata["messages"] }} | {{ =bdata["rps"] }} ({{ =round(bdata["rps"] / dcmp * 100, 1) }}%) | {{ =f"{bdata['latency_mean']}ms" }} | {{ =f"{bdata['latency_percentiles'][-2][1]}ms" }} | {{ =bdata["latency_std"] }} |
{{ pass }}
