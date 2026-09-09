//! Strict canonical tab-separated CMT1. Text is UTF-8 encoded as lowercase hex.
use crate::{Id, Memory, Op, Trace, Value, hex};
pub fn unhex(s: &str) -> Result<Vec<u8>, String> {
    if !s.len().is_multiple_of(2)
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("noncanonical hex".into());
    }
    s.as_bytes()
        .chunks_exact(2)
        .map(|p| {
            u8::from_str_radix(std::str::from_utf8(p).map_err(|e| e.to_string())?, 16)
                .map_err(|e| e.to_string())
        })
        .collect()
}
fn text(s: &str) -> Result<String, String> {
    String::from_utf8(unhex(s)?).map_err(|e| e.to_string())
}
pub fn id(s: &str) -> Result<Id, String> {
    unhex(s)?.try_into().map_err(|_| "UUID length".into())
}
pub fn value(v: &Value) -> String {
    match v {
        Value::Text(s) => format!("s{}", hex(s.as_bytes())),
        Value::Int(n) => format!("i{n}"),
        Value::Bool(b) => format!("b{}", u8::from(*b)),
        Value::Tags(tags) => format!(
            "l{}:{}",
            tags.len(),
            tags.iter()
                .map(|s| hex(s.as_bytes()))
                .collect::<Vec<_>>()
                .join(",")
        ),
    }
}
pub fn parse_value(s: &str) -> Result<Value, String> {
    let v = match s.as_bytes().first() {
        Some(b's') => Value::Text(text(&s[1..])?),
        Some(b'i') => Value::Int(s[1..].parse::<i64>().map_err(|e| e.to_string())?),
        Some(b'b') if s == "b0" || s == "b1" => Value::Bool(s == "b1"),
        Some(b'l') => {
            let (n, tail) = s[1..].split_once(':').ok_or("tag count")?;
            let n: usize = n.parse().map_err(|_| "tag count")?;
            let tags = if n == 0 && tail.is_empty() {
                vec![]
            } else {
                tail.split(',').map(text).collect::<Result<Vec<_>, _>>()?
            };
            if tags.len() != n {
                return Err("tag count mismatch".into());
            }
            Value::Tags(tags)
        }
        _ => return Err("unknown value".into()),
    };
    if value(&v) != s {
        return Err("noncanonical value".into());
    }
    Ok(v)
}
pub fn memory(m: &Memory) -> String {
    format!(
        "{}\t{}",
        hex(&m.id),
        m.fields.iter().map(value).collect::<Vec<_>>().join("\t")
    )
}
pub fn parse_memory(s: &str) -> Result<Memory, String> {
    let parts: Vec<_> = s.split('\t').collect();
    if parts.len() != 14 {
        return Err("Memory field count".into());
    }
    let m = Memory {
        id: id(parts[0])?,
        fields: parts[1..]
            .iter()
            .map(|s| parse_value(s))
            .collect::<Result<Vec<_>, _>>()?
            .try_into()
            .map_err(|_| "fields")?,
    };
    m.validate()?;
    Ok(m)
}
pub fn operation(op: &Op) -> String {
    match op {
        Op::Insert(m) => format!("insert\t{}", memory(m)),
        Op::Replace(m) => format!("replace\t{}", memory(m)),
        Op::Get(id) => format!("get\t{}", hex(id)),
        Op::Delete(id) => format!("delete\t{}", hex(id)),
        Op::Update(id, v) => format!("update\t{}\t{v}", hex(id)),
        Op::Guard {
            record,
            field,
            equals,
        } => format!("guard\t{field}\t{}\t{}", value(equals), memory(record)),
        Op::Equal(s) => format!("equal\t{}", hex(s.as_bytes())),
        Op::Aggregate => "aggregate".into(),
        Op::Page { after, limit } => format!(
            "page\t{limit}\t{}",
            after.map_or_else(|| "none".into(), |(t, id)| format!("{t}\t{}", hex(&id)))
        ),
    }
}
fn parse_op(s: &str) -> Result<Op, String> {
    let p: Vec<_> = s.split('\t').collect();
    let op = match p[0] {
        "insert" | "replace" if p.len() == 15 => {
            let m = parse_memory(&p[1..].join("\t"))?;
            if p[0] == "insert" {
                Op::Insert(m)
            } else {
                Op::Replace(m)
            }
        }
        "get" if p.len() == 2 => Op::Get(id(p[1])?),
        "delete" if p.len() == 2 => Op::Delete(id(p[1])?),
        "update" if p.len() == 3 => {
            let n = p[2].parse::<i64>().map_err(|_| "update integer")?;
            if n < 0 {
                return Err("negative count".into());
            }
            Op::Update(id(p[1])?, n)
        }
        "guard" if p.len() == 17 => {
            let field = p[1].parse::<usize>().map_err(|_| "guard field")?;
            if field >= 13 {
                return Err("guard field".into());
            }
            let record = parse_memory(&p[3..].join("\t"))?;
            let equals = parse_value(p[2])?;
            if std::mem::discriminant(&record.fields[field]) != std::mem::discriminant(&equals) {
                return Err("guard type".into());
            }
            Op::Guard {
                record,
                field,
                equals,
            }
        }
        "equal" if p.len() == 2 => Op::Equal(text(p[1])?),
        "aggregate" if p.len() == 1 => Op::Aggregate,
        "page" if p.len() == 3 || p.len() == 4 => {
            let limit = p[1].parse().map_err(|_| "page limit")?;
            if limit == 0 {
                return Err("zero page limit".into());
            }
            let after = if p.len() == 3 && p[2] == "none" {
                None
            } else if p.len() == 4 {
                Some((p[2].parse().map_err(|_| "page timestamp")?, id(p[3])?))
            } else {
                return Err("cursor".into());
            };
            Op::Page { after, limit }
        }
        _ => return Err("unknown operation or arity".into()),
    };
    if operation(&op) != s {
        return Err("noncanonical operation".into());
    }
    Ok(op)
}
pub fn encode(t: &Trace) -> String {
    let mut s = format!(
        "CMT1\t1\t{}\t{}\t{}\t{}\t{}\n",
        t.name,
        t.seed,
        t.records,
        t.ops.len(),
        t.shape
            .iter()
            .map(usize::to_string)
            .collect::<Vec<_>>()
            .join("\t")
    );
    for (id, op) in &t.ops {
        s.push_str(&format!("{id}\t{}\n", operation(op)));
    }
    s
}
pub fn decode(s: &str) -> Result<Trace, String> {
    let mut lines = s.lines();
    let h: Vec<_> = lines.next().ok_or("header")?.split('\t').collect();
    if h.len() != 11 || h[0] != "CMT1" || h[1] != "1" {
        return Err("CMT1 header/version".into());
    }
    if h[2].is_empty() || !h[2].bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-') {
        return Err("trace name".into());
    }
    let num = |s: &str| s.parse::<usize>().map_err(|_| "header integer".to_string());
    let mut t = Trace {
        name: h[2].into(),
        seed: h[3].parse().map_err(|_| "seed")?,
        records: num(h[4])?,
        shape: [num(h[6])?, num(h[7])?, num(h[8])?, num(h[9])?, num(h[10])?],
        ops: vec![],
    };
    if t.shape[0] > t.shape[1] || t.shape[2] > t.shape[3] {
        return Err("shape bounds".into());
    }
    for (index, line) in lines.enumerate() {
        let (id, op) = line.split_once('\t').ok_or("operation line")?;
        let id = id.parse::<u64>().map_err(|_| "operation id")?;
        if id != index as u64 + 1 {
            return Err("nonsequential operation id".into());
        }
        t.ops.push((id, parse_op(op)?));
    }
    if t.ops.len() != num(h[5])? || encode(&t) != s {
        return Err("count or noncanonical encoding".into());
    }
    let distinct: std::collections::BTreeSet<_> = t
        .ops
        .iter()
        .filter_map(|(_, op)| {
            if let Op::Insert(m) = op {
                Some(m.id)
            } else {
                None
            }
        })
        .collect();
    if distinct.len() != t.records {
        return Err("distinct insert record count".into());
    }
    Ok(t)
}
