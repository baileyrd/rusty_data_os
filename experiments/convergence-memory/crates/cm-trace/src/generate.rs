use crate::{Id, Memory, Op, Trace, Value};
pub fn uuid(n: u64) -> Id {
    let mut id = [0; 16];
    id[0] = 0xc1;
    id[6] = 0x40;
    id[8] = 0x80;
    id[9..].copy_from_slice(&n.to_be_bytes()[1..]);
    id
}
pub struct SplitMix64(pub u64);
impl SplitMix64 {
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }
}
pub fn record(n: u64, revision: i64, rng: &mut SplitMix64, realistic: bool) -> Memory {
    let content_len = if realistic {
        200 + (rng.next_u64() % 1801) as usize
    } else {
        12
    };
    let metadata_len = if realistic {
        50 + (rng.next_u64() % 451) as usize
    } else {
        12
    };
    let content = (0..content_len)
        .map(|_| (b'a' + (rng.next_u64() % 26) as u8) as char)
        .collect();
    let tags = (0..rng.next_u64() % 9)
        .map(|i| format!("tag-{i}-{revision}"))
        .collect();
    Memory {
        id: uuid(n),
        fields: [
            Value::Text(content),
            Value::Text(format!("category-{}", n % 7)),
            Value::Tags(tags),
            Value::Text(format!("source-{revision}")),
            Value::Text(format!("{{\"v\":\"{}\"}}", "x".repeat(metadata_len - 8))),
            Value::Int(1_700_000_000_000 + n as i64),
            Value::Int(1_800_000_000_000 + (n % 4) as i64 + revision),
            Value::Text(format!("type-{revision}")),
            Value::Text(format!("status-{revision}")),
            Value::Bool(revision % 2 == 1),
            Value::Int(revision),
            Value::Int(if revision % 2 == 1 { 100 + revision } else { 0 }),
            Value::Text(format!("node-{revision}")),
        ],
    }
}
fn push(t: &mut Trace, op: Op) {
    t.ops.push((t.ops.len() as u64 + 1, op));
}
fn queries(t: &mut Trace, n: u64) {
    push(t, Op::Get(uuid(n)));
    push(t, Op::Equal(format!("category-{}", n % 7)));
    push(t, Op::Aggregate);
    push(
        t,
        Op::Page {
            after: None,
            limit: 3,
        },
    );
    push(
        t,
        Op::Page {
            after: Some((1_800_000_000_000, uuid(4))),
            limit: 3,
        },
    );
}
pub fn small() -> Trace {
    let mut t = Trace {
        name: "small-deterministic".into(),
        seed: 7,
        records: 4,
        shape: [12, 12, 12, 12, 8],
        ops: vec![],
    };
    let mut rng = SplitMix64(t.seed);
    for n in [4, 8, 1, 2] {
        push(&mut t, Op::Insert(record(n, 0, &mut rng, false)));
    }
    queries(&mut t, 4);
    push(&mut t, Op::Insert(record(4, 9, &mut rng, false)));
    queries(&mut t, 4);
    push(&mut t, Op::Update(uuid(4), 17));
    queries(&mut t, 4);
    push(&mut t, Op::Replace(record(4, 1, &mut rng, false)));
    queries(&mut t, 4);
    push(
        &mut t,
        Op::Guard {
            record: record(4, 2, &mut rng, false),
            field: 10,
            equals: Value::Int(999),
        },
    );
    queries(&mut t, 4);
    push(
        &mut t,
        Op::Guard {
            record: record(4, 2, &mut rng, false),
            field: 10,
            equals: Value::Int(1),
        },
    );
    queries(&mut t, 4);
    let mut text_guard_record = record(4, 2, &mut rng, false);
    text_guard_record.fields[8] = Value::Text("status-text-guard".into());
    push(
        &mut t,
        Op::Guard {
            record: text_guard_record.clone(),
            field: 8,
            equals: Value::Text("wrong-status".into()),
        },
    );
    push(&mut t, Op::Get(uuid(4)));
    push(
        &mut t,
        Op::Guard {
            record: text_guard_record,
            field: 8,
            equals: Value::Text("status-2".into()),
        },
    );
    push(&mut t, Op::Get(uuid(4)));
    push(&mut t, Op::Delete(uuid(4)));
    queries(&mut t, 4);
    push(&mut t, Op::Insert(record(4, 3, &mut rng, false)));
    queries(&mut t, 4);
    push(&mut t, Op::Update(uuid(99), 1));
    push(&mut t, Op::Replace(record(99, 1, &mut rng, false)));
    push(
        &mut t,
        Op::Guard {
            record: record(99, 2, &mut rng, false),
            field: 10,
            equals: Value::Int(1),
        },
    );
    push(&mut t, Op::Delete(uuid(99)));
    t
}
pub fn repeated(n: usize, seed: u64) -> Trace {
    let mut t = Trace {
        name: format!("repeated-mutations-{}k", n / 1000),
        seed,
        records: n,
        shape: [200, 2000, 50, 500, 8],
        ops: vec![],
    };
    let mut rng = SplitMix64(seed);
    for id in 1..=n as u64 {
        push(&mut t, Op::Insert(record(id, 0, &mut rng, true)));
    }
    queries(&mut t, 1);
    for round in 1..=2 {
        // Permutation by Fisher-Yates: every live identity is repeatedly mutated.
        let mut ids: Vec<_> = (1..=n as u64).collect();
        for i in (1..ids.len()).rev() {
            let j = (rng.next_u64() % (i as u64 + 1)) as usize;
            ids.swap(i, j);
        }
        for id in ids {
            push(&mut t, Op::Update(uuid(id), round * 10));
            push(&mut t, Op::Replace(record(id, round, &mut rng, true)));
            push(
                &mut t,
                Op::Guard {
                    record: record(id, round + 2, &mut rng, true),
                    field: 10,
                    equals: Value::Int(-1),
                },
            );
            push(
                &mut t,
                Op::Guard {
                    record: record(id, round + 2, &mut rng, true),
                    field: 10,
                    equals: Value::Int(round),
                },
            );
            push(
                &mut t,
                Op::Guard {
                    record: record(id, round + 2, &mut rng, true),
                    field: 8,
                    equals: Value::Text(format!("status-{}", round + 2)),
                },
            );
            push(&mut t, Op::Delete(uuid(id)));
            push(&mut t, Op::Insert(record(id, round + 3, &mut rng, true)));
            if id % 1000 == 0 {
                queries(&mut t, id);
            }
        }
        queries(&mut t, 1);
    }
    t
}
