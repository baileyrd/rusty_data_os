"""Independent fixture arithmetic over literal CMT1, using Python hashlib.

Run from this directory. This does not invoke Rust or either database.
The Rust hand assertions separately audit critical small-trace outcomes.
"""
from pathlib import Path
import hashlib


def digest(data):
    return hashlib.sha256(data.encode()).hexdigest()


trace = Path(__file__).with_name('small.cmt')
state = {}
signatures = []
for line in trace.read_text().splitlines()[1:]:
    op_id, kind, *args = line.split('\t')
    outcome, rows, aggregate = '', [], 'None'
    if kind == 'insert':
        outcome = 'Duplicate' if args[0] in state else 'Inserted'
        if outcome == 'Inserted':
            state[args[0]] = args[1:]
    elif kind == 'replace':
        outcome = 'Replaced' if args[0] in state else 'NotFound'
        if outcome == 'Replaced':
            state[args[0]] = args[1:]
    elif kind == 'guard':
        field, equals, key, *fields = args
        outcome = ('NotFound' if key not in state else
                   'GuardFailed' if state[key][int(field)] != equals else 'Replaced')
        if outcome == 'Replaced':
            state[key] = fields
    elif kind == 'update':
        outcome = 'Updated' if args[0] in state else 'NotFound'
        if outcome == 'Updated':
            state[args[0]][10] = 'i' + args[1]
    elif kind == 'delete':
        outcome = 'Deleted' if args[0] in state else 'NotFound'
        state.pop(args[0], None)
    elif kind == 'get':
        outcome = 'Rows' if args[0] in state else 'NotFound'
        rows = [args[0]] if args[0] in state else []
    elif kind == 'equal':
        outcome = 'Rows'
        rows = sorted(k for k, fields in state.items() if fields[1] == 's' + args[0])
    elif kind == 'page':
        outcome = 'Rows'
        pairs = sorted((int(f[6][1:]), k) for k, f in state.items())
        if args[1] != 'none':
            pairs = [p for p in pairs if p > (int(args[1]), args[2])]
        rows = [k for _, k in pairs[:int(args[0])]]
    elif kind == 'aggregate':
        outcome = 'Aggregate'
        counts = [int(f[10][1:]) for f in state.values()]
        lo, hi = (f'Some({min(counts)})', f'Some({max(counts)})') if counts else ('None', 'None')
        aggregate = f'Some(({len(counts)}, {sum(counts)}, {lo}, {hi}))'
    else:
        raise ValueError(kind)
    digests = ','.join(k + ':' + digest('CMT1/record\t' + '\t'.join([k] + state[k])) for k in sorted(rows))
    signatures.append('\t'.join([outcome, aggregate, digests, ','.join(rows)]))

assert len(signatures) == 59
assert len(state) == 4
assert sum(int(f[10][1:]) for f in state.values()) == 3
output = [f'CMT1-results\t1\t{hashlib.sha256(trace.read_bytes()).hexdigest()}\t{b"independent-python-fixture".hex()}\t59\t4',
          'durability\t' + b'independent model; no engine writes or durability claim'.hex()]
output += [f'op\t{i}\t{s.encode().hex()}' for i, s in enumerate(signatures, 1)]
output += [f'record\t{k}\t' + digest('CMT1/record\t' + '\t'.join([k] + state[k])) for k in sorted(state)]
for i in range(13):
    data = f'CMT1/column/{i}\n' + ''.join(f'{k}\t{state[k][i]}\n' for k in sorted(state))
    output.append(f'column\t{i}\t{digest(data)}')
result = trace.with_name('small.results.cmt')
result.write_bytes(('\n'.join(output) + '\n').encode())
trace.with_name('SHA256SUMS').write_bytes(''.join(f'{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}\n' for p in [trace, result]).encode())
