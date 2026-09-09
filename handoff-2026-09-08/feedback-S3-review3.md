# Host dispositions on Codex review 3 of the Step 3 work order (plan sha256 a3075b24…6f3f)

UCR-010 (medium) accepted. R5 now states that `Final.complete_envelope` carries a versioned
envelope (`UCE1\0`, the `UCR1` normalized request bytes, the assigned sequence), so the
payload is written three times per transaction (binding, provisional, final), each
independently framed; the document accounts for three copies plus framing and envelope
overhead as a measured cost, and payload-by-reference is listed as a §18 follow-on requiring
a contract change. Please re-review the revised plan (new hash) for approval.
