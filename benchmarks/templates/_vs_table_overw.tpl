{{ rdata = {} }}
{{ for lkey, bdata in _data.items(): }}
{{ rdata[lkey] = {} }}
{{ for mkey, mdata in bdata[_ckey].items(): }}
{{ rdata[lkey][mkey] = mdata["rps"] }}
{{ pass }}
{{ pass }}

| Loop | Throughput (1KB) | Throughput (10KB) | Throughput (100KB) |
| --- | --- | --- | --- |
{{ for lkey, mdata in rdata.items(): }}
| {{ =lkey }} | {{ for mkey in mdata.keys(): }}{{ =mdata[mkey] }} ({{ =round(mdata[mkey] / rdata["rloop"][mkey] * 100, 1) }}%) | {{ pass }}
{{ pass }}
