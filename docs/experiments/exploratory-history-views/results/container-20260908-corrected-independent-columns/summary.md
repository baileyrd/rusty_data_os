# Exploratory history/materialization run summary

All measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.

| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 34765 (34262–62417) | 116548 (95434–196707) | 129911 (108478–1341462) | 4310 (3699–5266) | 7092 (7020–8373) | 1844 (1717–2032) | 3711 (3538–4321) |
| 1000 | 318599 (316971–1660826) | 751627 (732414–843096) | 6953200 (6248823–8389986) | 28707 (26684–65238) | 29313 (28397–38495) | 11973 (11837–12382) | 13245 (12837–40815) |
| 10000 | 3262352 (3216913–3508448) | 6933429 (6876955–11875935) | 547478309 (529186476–589000460) | 372203 (285952–477924) | 279586 (253555–314708) | 110869 (109766–133390) | 101650 (101147–121492) |
