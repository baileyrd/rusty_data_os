# Exploratory history/materialization run summary

All measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.

| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 34081 (33411–35274) | 94778 (89278–112006) | 62490 (45804–107879) | 3823 (3445–5091) | 6310 (5963–8160) | 1768 (1698–4312) | 3542 (3421–3725) |
| 1000 | 328249 (312251–366653) | 702609 (662154–740019) | 379115 (160368–535545) | 26330 (25742–26609) | 27415 (27193–29536) | 11726 (11477–11820) | 12467 (11994–12838) |
| 10000 | 3444466 (3320246–3548878) | 7720862 (6595909–8405969) | 2502494 (2275321–17027527) | 302988 (280100–1008672) | 276417 (237536–343186) | 111800 (109168–166968) | 101190 (100529–153008) |
