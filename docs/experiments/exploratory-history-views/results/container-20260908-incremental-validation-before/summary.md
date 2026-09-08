# Exploratory history/materialization run summary

All measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.

| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 35092 (33935–75136) | 111780 (85119–123877) | 119032 (103202–145677) | 3960 (3550–56289) | 6302 (6203–10537) | 1829 (1758–3378) | 3569 (3463–5101) |
| 1000 | 317105 (315609–413441) | 731644 (688199–778549) | 5584829 (5545372–6084391) | 26474 (26059–28672) | 27803 (27672–29205) | 11718 (11604–11806) | 12746 (12631–12938) |
| 10000 | 3480870 (3210914–4793742) | 6701669 (6463946–9045934) | 542289996 (526597487–567061188) | 334334 (311209–426990) | 273553 (261550–321716) | 142274 (109146–152376) | 103000 (100120–247016) |
