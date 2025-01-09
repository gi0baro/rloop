| Loop | Total requests | Throughput | Mean latency | 99p latency | Latency stdev |
| --- | --- | --- | --- | --- | --- |
{{ dcmp = _data["rloop"][_ckey][_dkey]["rps"] }}
{{ for lkey, bdata in _data.items(): }}
{{ lbdata = bdata[_ckey][_dkey] }}
| {{ =lkey }} | {{ =lbdata["messages"] }} | {{ =lbdata["rps"] }} ({{ =round(lbdata["rps"] / dcmp * 100, 1) }}%) | {{ =f"{lbdata['latency_mean']}ms" }} | {{ =f"{lbdata['latency_percentiles'][-1][1]}ms" }} | {{ =lbdata["latency_std"] }} |
{{ pass }}
