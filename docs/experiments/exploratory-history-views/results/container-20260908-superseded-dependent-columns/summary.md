# Exploratory history/materialization run summary

All measured trials passed correctness. Values are median nanoseconds with observed min–max across five samples; five samples do not support tail-latency claims.

| Events | Encode ns | Append ns | Replay ns | Row rebuild ns | Column rebuild ns | Direct row ns | Direct column ns |
|---:|---:|---:|---:|---:|---:|---:|---:|
| 100 | 32266 (31635–36037) | 98000 (87650–218306) | 92736 (65014–139331) | 3461 (3249–5487) | 3603 (3507–5442) | 1460 (1435–2513) | 1798 (1676–2970) |
| 1000 | 284747 (282562–330104) | 760609 (714294–843221) | 5525902 (4988092–5948060) | 26718 (26330–51313) | 26017 (25815–26911) | 10474 (10446–10637) | 10398 (10354–10521) |
| 10000 | 3161290 (2985360–4101595) | 6677593 (6512125–7655897) | 459493400 (442619612–497127411) | 311475 (290552–492855) | 260377 (257344–304794) | 124650 (100658–166419) | 97350 (97038–117081) |
