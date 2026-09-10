//! Frozen Step 5 synthetic fidelity proof; never a production importer.
//! include_deleted=true deliberately keeps tombstones/superseded rows live.
//! Source: remind_me_core models.rs Memory and export.rs mixed JSON envelope.
//! All 28 explicitly named Memory fields are preserved (the order says 27).
mod support;

use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    net::{Shutdown, SocketAddr, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    thread::JoinHandle,
    time::Duration,
};
use uc_core::{CheckpointRef, Durability, OpenReport, Position};
use uc_facade::{EntityStore, LoopbackListener, MemoryStore, RelationStore, serve};
use uc_protocol::{codec::*, framing::*, *};

// Test-local JSON value/parser keeps the frozen dependency-free workspace intact.
// Numbers retain their JSON lexeme: no float rounding during stash reconstruction.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Json {
    Null,
    Bool(bool),
    Number(String),
    Str(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
}
impl Json {
    fn object(&self) -> &BTreeMap<String, Json> {
        let Self::Object(v) = self else {
            panic!("expected object: {self:?}")
        };
        v
    }
    fn array(&self) -> &[Json] {
        let Self::Array(v) = self else {
            panic!("expected array: {self:?}")
        };
        v
    }
    fn string(&self) -> &str {
        let Self::Str(v) = self else {
            panic!("expected string: {self:?}")
        };
        v
    }
    fn get(&self, key: &str) -> &Json {
        &self.object()[key]
    }
    fn canonical(&self) -> String {
        match self {
            Self::Null => "null".into(),
            Self::Bool(v) => v.to_string(),
            Self::Number(v) => v.clone(),
            Self::Str(v) => quote(v),
            Self::Array(v) => format!(
                "[{}]",
                v.iter().map(Self::canonical).collect::<Vec<_>>().join(",")
            ),
            Self::Object(v) => format!(
                "{{{}}}",
                v.iter()
                    .map(|(k, v)| format!("{}:{}", quote(k), v.canonical()))
                    .collect::<Vec<_>>()
                    .join(",")
            ),
        }
    }
}
fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
struct Parser<'a>(&'a str);
impl<'a> Parser<'a> {
    fn ws(&mut self) {
        self.0 = self.0.trim_start_matches([' ', '\n', '\r', '\t']);
    }
    fn take(&mut self, text: &str) -> Result<(), String> {
        self.0 = self
            .0
            .strip_prefix(text)
            .ok_or_else(|| format!("expected {text}"))?;
        Ok(())
    }
    fn character(&mut self) -> Result<char, String> {
        let c = self.0.chars().next().ok_or("truncated JSON")?;
        self.0 = &self.0[c.len_utf8()..];
        Ok(c)
    }
    fn hex4(&mut self) -> Result<u32, String> {
        let mut n = 0;
        for _ in 0..4 {
            n = n * 16 + self.character()?.to_digit(16).ok_or("bad Unicode escape")?;
        }
        Ok(n)
    }
    fn string(&mut self) -> Result<String, String> {
        self.take("\"")?;
        let mut out = String::new();
        loop {
            match self.character()? {
                '"' => return Ok(out),
                '\\' => {
                    let c = match self.character()? {
                        '"' => '"',
                        '\\' => '\\',
                        '/' => '/',
                        'b' => '\u{8}',
                        'f' => '\u{c}',
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        'u' => {
                            let mut n = self.hex4()?;
                            if (0xd800..=0xdbff).contains(&n) {
                                self.take("\\u")?;
                                let low = self.hex4()?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err("bad surrogate".into());
                                }
                                n = 0x10000 + (n - 0xd800) * 1024 + low - 0xdc00;
                            }
                            char::from_u32(n).ok_or("bad Unicode scalar")?
                        }
                        _ => return Err("bad JSON escape".into()),
                    };
                    out.push(c);
                }
                c if c < ' ' => return Err("unescaped control".into()),
                c => out.push(c),
            }
        }
    }
    fn digits(&mut self) -> Result<(), String> {
        let start = self.0.len();
        self.0 = self.0.trim_start_matches(|c: char| c.is_ascii_digit());
        if start == self.0.len() {
            return Err("expected digits".into());
        }
        Ok(())
    }
    fn number(&mut self) -> Result<Json, String> {
        let start = self.0;
        if self.0.starts_with('-') {
            self.take("-")?;
        }
        if self.0.starts_with('0') {
            self.take("0")?;
        } else {
            self.digits()?;
        }
        if self.0.starts_with('.') {
            self.take(".")?;
            self.digits()?;
        }
        if self.0.starts_with(['e', 'E']) {
            self.character()?;
            if self.0.starts_with(['+', '-']) {
                self.character()?;
            }
            self.digits()?;
        }
        Ok(Json::Number(start[..start.len() - self.0.len()].into()))
    }
    fn value(&mut self, depth: usize) -> Result<Json, String> {
        if depth > 64 {
            return Err("JSON depth limit".into());
        }
        self.ws();
        match self.0.chars().next().ok_or("missing value")? {
            'n' => {
                self.take("null")?;
                Ok(Json::Null)
            }
            't' => {
                self.take("true")?;
                Ok(Json::Bool(true))
            }
            'f' => {
                self.take("false")?;
                Ok(Json::Bool(false))
            }
            '"' => Ok(Json::Str(self.string()?)),
            '[' => {
                self.take("[")?;
                self.ws();
                let mut values = Vec::new();
                if !self.0.starts_with(']') {
                    loop {
                        values.push(self.value(depth + 1)?);
                        self.ws();
                        if self.0.starts_with(']') {
                            break;
                        }
                        self.take(",")?;
                    }
                }
                self.take("]")?;
                Ok(Json::Array(values))
            }
            '{' => {
                self.take("{")?;
                self.ws();
                let mut values = BTreeMap::new();
                if !self.0.starts_with('}') {
                    loop {
                        self.ws();
                        let key = self.string()?;
                        self.ws();
                        self.take(":")?;
                        if values.insert(key, self.value(depth + 1)?).is_some() {
                            return Err("duplicate key".into());
                        }
                        self.ws();
                        if self.0.starts_with('}') {
                            break;
                        }
                        self.take(",")?;
                    }
                }
                self.take("}")?;
                Ok(Json::Object(values))
            }
            '-' | '0'..='9' => self.number(),
            _ => Err("invalid JSON value".into()),
        }
    }
}
fn parse(text: &str) -> Result<Json, String> {
    let mut p = Parser(text);
    let value = p.value(0)?;
    p.ws();
    if !p.0.is_empty() {
        return Err("trailing JSON".into());
    }
    Ok(value)
}
fn obj(entries: impl IntoIterator<Item = (String, Json)>) -> Json {
    Json::Object(entries.into_iter().collect())
}
fn subset(record: &Json, keys: &[&str]) -> Json {
    obj(keys.iter().map(|k| ((*k).into(), record.get(k).clone())))
}

// Strict RFC3339 calendar/offset validation; fractions floor, including pre-epoch.
// Leap-second :60 is rejected explicitly: the target POSIX-ms field has no such instant.
fn milliseconds(text: &str) -> Result<i64, String> {
    fn number(s: &str) -> Result<i64, String> {
        if s.is_empty() || !s.bytes().all(|b| b.is_ascii_digit()) {
            return Err("timestamp digits".into());
        }
        s.parse().map_err(|_| "timestamp overflow".into())
    }
    if !text.is_ascii() || text.len() < 20 {
        return Err("timestamp shape".into());
    }
    for (i, c) in [(4, b'-'), (7, b'-'), (13, b':'), (16, b':')] {
        if text.as_bytes()[i] != c {
            return Err("timestamp separator".into());
        }
    }
    if !matches!(text.as_bytes()[10], b'T' | b't') {
        return Err("timestamp T".into());
    }
    let (mut y, m, d) = (
        number(&text[..4])?,
        number(&text[5..7])?,
        number(&text[8..10])?,
    );
    let (h, min, s) = (
        number(&text[11..13])?,
        number(&text[14..16])?,
        number(&text[17..19])?,
    );
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    if d < 1 || d > days || h > 23 || min > 59 || s > 59 {
        return Err("timestamp range (leap seconds unsupported)".into());
    }
    let mut rest = &text[19..];
    let mut frac = 0;
    if let Some(tail) = rest.strip_prefix('.') {
        let n = tail.bytes().take_while(u8::is_ascii_digit).count();
        if n == 0 {
            return Err("timestamp fraction".into());
        }
        let digits = &tail[..n];
        for i in 0..3 {
            frac = frac * 10 + i64::from(digits.as_bytes().get(i).copied().unwrap_or(b'0') - b'0');
        }
        rest = &tail[n..];
    }
    let offset = if matches!(rest, "Z" | "z") {
        0
    } else {
        if rest.len() != 6 || !rest.starts_with(['+', '-']) || rest.as_bytes()[3] != b':' {
            return Err("timestamp zone".into());
        }
        let (h, m) = (number(&rest[1..3])?, number(&rest[4..6])?);
        if h > 23 || m > 59 {
            return Err("timestamp zone range".into());
        }
        (h * 60 + m) * 60 * if rest.starts_with('-') { -1 } else { 1 }
    };
    // Proleptic Gregorian civil-date conversion, March-based 400-year eras.
    y -= i64::from(m <= 2);
    let era = y.div_euclid(400);
    let yo = y - era * 400;
    let mp = m + if m > 2 { -3 } else { 9 };
    let day = era * 146097 + yo * 365 + yo / 4 - yo / 100 + (153 * mp + 2) / 5 + d - 1 - 719468;
    Ok((day * 86400 + h * 3600 + min * 60 + s - offset) * 1000 + frac)
}
fn mapped_id(source: &str, memory: bool) -> Result<RecordId, String> {
    let hex = if memory {
        source.strip_prefix("mem_").ok_or("missing mem_ prefix")?
    } else {
        source
    };
    let size = if memory { 16 } else { 6 };
    if hex.len() != size * 2 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err("invalid id hex/length".into());
    }
    let mut bytes = [0; 16];
    for (i, slot) in bytes[..size].iter_mut().enumerate() {
        *slot = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16).map_err(|e| e.to_string())?;
    }
    Ok(RecordId(bytes))
}
fn original_id(id: RecordId, memory: bool) -> String {
    if memory {
        format!("mem_{}", cm_trace::hex(&id.0))
    } else {
        assert_eq!(&id.0[6..], &[0; 10]);
        cm_trace::hex(&id.0[..6])
    }
}
fn normalized(name: &str) -> String {
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
fn entity_identity(name: &str) -> String {
    cm_trace::hex(&cm_trace::sha256(normalized(name).as_bytes()))[..12].into()
}

const EXTRA: [&str; 21] = [
    "id",
    "capture_id",
    "subject",
    "predicate",
    "object",
    "superseded_by",
    "decay_rate",
    "vitality",
    "base_weight",
    "accessed_at",
    "doc_id",
    "chunk_index",
    "remind_at",
    "client",
    "source_capture_id",
    "memory_type",
    "status",
    "node_id",
    "created_at",
    "updated_at",
    "deleted_at",
];
const DIRECT: [(&str, FieldRef); 6] = [
    ("content", 0),
    ("category", 1),
    ("tags", 2),
    ("source", 3),
    ("sensitive", 9),
    ("access_count", 10),
];
fn scan(value: &Json) -> ScanValue {
    match value {
        Json::Str(s) => ScanValue::Str(s.clone()),
        Json::Number(n) => ScanValue::I64(n.parse().unwrap()),
        Json::Bool(b) => ScanValue::Bool(*b),
        Json::Array(a) => ScanValue::StrList(a.iter().map(|s| s.string().into()).collect()),
        _ => panic!("not a projected field"),
    }
}
fn unscan(value: &ScanValue) -> Json {
    match value {
        ScanValue::Str(s) => Json::Str(s.clone()),
        ScanValue::I64(n) => Json::Number(n.to_string()),
        ScanValue::Bool(b) => Json::Bool(*b),
        ScanValue::StrList(a) => Json::Array(a.iter().cloned().map(Json::Str).collect()),
        _ => panic!("unexpected projected type"),
    }
}
fn option_projection(v: &Json, default: &str) -> ScanValue {
    ScanValue::Str(if *v == Json::Null {
        default.into()
    } else {
        v.string().into()
    })
}
fn field(fields: &Fields, tag: FieldRef) -> &ScanValue {
    &fields.iter().find(|(t, _)| *t == tag).unwrap().1
}
fn memory_fields(source: &Json) -> Result<(RecordId, Fields), String> {
    let id = mapped_id(source.get("id").string(), true)?;
    let envelope = obj([
        ("original_metadata".into(), source.get("metadata").clone()),
        ("_remind_me_migration_extra".into(), subset(source, &EXTRA)),
    ]);
    let mut fields: Fields = DIRECT
        .iter()
        .map(|(k, t)| (*t, scan(source.get(k))))
        .collect();
    fields.extend([
        (4, ScanValue::Str(envelope.canonical())),
        (
            5,
            ScanValue::I64(milliseconds(source.get("created_at").string())?),
        ),
        (
            6,
            ScanValue::I64(milliseconds(source.get("updated_at").string())?),
        ),
        (
            7,
            option_projection(source.get("memory_type"), "unclassified"),
        ),
        (8, option_projection(source.get("status"), "active")),
        (
            11,
            ScanValue::I64(if *source.get("deleted_at") == Json::Null {
                0
            } else {
                milliseconds(source.get("deleted_at").string())?
            }),
        ),
        (12, option_projection(source.get("node_id"), "")),
    ]);
    fields.sort_by_key(|(tag, _)| *tag);
    Ok((id, fields))
}
fn reconstruct_memory(id: RecordId, fields: &Fields) -> Json {
    let ScanValue::Str(encoded) = field(fields, 4) else {
        panic!("envelope type")
    };
    let envelope = parse(encoded).unwrap();
    assert_eq!(envelope.object().len(), 2);
    let mut out = envelope.get("_remind_me_migration_extra").object().clone();
    assert_eq!(out.len(), 21);
    assert_eq!(mapped_id(out["id"].string(), true).unwrap(), id);
    for (key, tag) in DIRECT {
        assert!(out.insert(key.into(), unscan(field(fields, tag))).is_none());
    }
    out.insert("metadata".into(), envelope.get("original_metadata").clone());
    assert_eq!(out.len(), 28);
    Json::Object(out)
}

#[derive(Clone, Debug)]
struct Fixture {
    memories: Vec<Json>,
    entities: Vec<Json>,
    relations: Vec<Json>,
    links: Vec<Json>,
}
impl Fixture {
    fn read() -> Self {
        let json = parse(include_str!("fixtures/remind-me/export.json")).unwrap();
        let mut f = Self {
            memories: vec![],
            entities: vec![],
            relations: vec![],
            links: vec![],
        };
        let mut graph = false;
        let mut entities = BTreeSet::new();
        for record in json.array() {
            let mut record = record.object().clone();
            match record.remove("record_type") {
                None => {
                    assert!(!graph);
                    assert_eq!(record.remove("role"), Some(Json::Str("assistant".into())));
                    assert_eq!(record.len(), 28);
                    f.memories.push(Json::Object(record));
                }
                Some(Json::Str(kind)) => {
                    graph = true;
                    let record = Json::Object(record);
                    match kind.as_str() {
                        "entity" => {
                            assert_eq!(record.object().len(), 6);
                            assert!(entities.insert(record.get("id").string().to_owned()));
                            f.entities.push(record);
                        }
                        "entity_relation" => {
                            assert_eq!(record.object().len(), 6);
                            for k in ["subject_entity_id", "object_entity_id"] {
                                assert!(entities.contains(record.get(k).string()));
                            }
                            f.relations.push(record);
                        }
                        "memory_entity" => {
                            assert_eq!(record.object().len(), 3);
                            assert!(entities.contains(record.get("entity_id").string()));
                            f.links.push(record);
                        }
                        _ => panic!("unexpected graph type"),
                    }
                }
                _ => panic!("invalid record_type"),
            }
        }
        f
    }
    fn mentions(&self, id: &str) -> i64 {
        self.links
            .iter()
            .filter(|l| l.get("entity_id").string() == id)
            .count() as i64
    }
    fn entity_fields(&self, e: &Json) -> Fields {
        vec![
            (0, scan(e.get("name"))),
            (1, option_projection(e.get("kind"), "")),
            (2, ScanValue::I64(self.mentions(e.get("id").string()))),
            (3, scan(e.get("aliases"))),
        ]
    }
}
fn relation_fields(r: &Json) -> Fields {
    vec![
        (0, scan(r.get("subject_entity_id"))),
        (1, scan(r.get("relation"))),
        (2, scan(r.get("object_entity_id"))),
        (
            3,
            ScanValue::I64(milliseconds(r.get("created_at").string()).unwrap()),
        ),
        (
            4,
            ScanValue::I64(milliseconds(r.get("updated_at").string()).unwrap()),
        ),
        (5, ScanValue::Str(String::new())),
        (6, ScanValue::I64(0)),
    ]
}

// These fields only prove import-time reading, never file-backup/replay fidelity.
struct Sidecar {
    entities: BTreeMap<String, Json>,
    relations: BTreeMap<String, Json>,
    links: BTreeMap<(String, String), Json>,
}
impl Sidecar {
    fn new() -> Self {
        Self {
            entities: BTreeMap::new(),
            relations: BTreeMap::new(),
            links: BTreeMap::new(),
        }
    }
    fn verify(self, fixture: &Fixture) {
        assert_eq!(self.entities.len(), fixture.entities.len());
        for r in &fixture.entities {
            assert_eq!(
                self.entities[r.get("id").string()],
                subset(r, &["kind", "created_at", "updated_at"])
            );
        }
        assert_eq!(self.relations.len(), fixture.relations.len());
        for r in &fixture.relations {
            assert_eq!(
                self.relations[r.get("id").string()],
                subset(r, &["created_at", "updated_at"])
            );
        }
        assert_eq!(self.links.len(), fixture.links.len());
        for r in &fixture.links {
            assert_eq!(
                &self.links[&(
                    r.get("memory_id").string().into(),
                    r.get("entity_id").string().into()
                )],
                r.get("created_at")
            );
        }
        // Consumed here. Later verification cannot access an import-time sidecar.
    }
}

struct Server {
    worker: Option<JoinHandle<std::io::Result<()>>>,
    stop: Arc<AtomicBool>,
    addr: SocketAddr,
}
impl Server {
    fn start(stores: Vec<Arc<dyn Store>>) -> Self {
        let registry = support::registry(&stores);
        assert!(registry.has_relationship_lock());
        let listener = LoopbackListener::bind().unwrap();
        let addr = listener.local_addr().unwrap();
        assert_eq!(addr.ip(), std::net::Ipv4Addr::LOCALHOST);
        let stop = Arc::new(AtomicBool::new(false));
        let signal = stop.clone();
        let worker = Some(std::thread::spawn(move || {
            serve(listener, registry, &signal)
        }));
        Self { worker, stop, addr }
    }
    fn client(&self) -> Client {
        let stream = TcpStream::connect(self.addr).unwrap();
        stream
            .set_read_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        stream
            .set_write_timeout(Some(Duration::from_secs(20)))
            .unwrap();
        stream.set_nodelay(true).unwrap();
        let mut c = Client(stream);
        assert_eq!(
            c.request(Request::Hello {
                protocol_version: 22
            }),
            Response::Hello {
                protocol_version: 22
            }
        );
        c
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            worker.join().unwrap().unwrap();
        }
    }
}
struct Client(TcpStream);
impl Client {
    fn request(&mut self, r: Request) -> Response {
        write_message(&mut self.0, &encode_request(&r)).unwrap();
        decode_response(&read_message(&mut self.0).unwrap()).unwrap()
    }
    fn ok(&mut self, r: Request) {
        assert_eq!(self.request(r), Response::Ok);
    }
    fn use_table(&mut self, name: &str) {
        self.ok(Request::Use { table: name.into() });
    }
    fn get(&mut self, id: RecordId) -> Fields {
        let Response::Record { id: actual, fields } = self.request(Request::GetById { id }) else {
            panic!("missing live record {id:?}")
        };
        assert_eq!(id, actual);
        fields
    }
}
impl Drop for Client {
    fn drop(&mut self) {
        self.0.shutdown(Shutdown::Both).unwrap();
    }
}

struct Engines {
    memory: uc_memory::MemoryEngine,
    entity: uc_entity::EntityEngine,
    relation: uc_relation::RelationEngine,
}
impl Engines {
    fn open(path: &Path, create: bool, expected: [Option<Position>; 3]) -> Self {
        let (memory, mr) = if create {
            uc_memory::MemoryEngine::create_with_label(
                &path.join("memory"),
                Durability::D1,
                "Step 5 synthetic D1 fidelity only".into(),
            )
        } else {
            uc_memory::MemoryEngine::open(&path.join("memory"), Durability::D1)
        }
        .unwrap();
        let (entity, er) = if create {
            uc_entity::EntityEngine::create(&path.join("entity"), Durability::D1)
        } else {
            uc_entity::EntityEngine::open(&path.join("entity"), Durability::D1)
        }
        .unwrap();
        let (relation, rr) = if create {
            uc_relation::RelationEngine::create(&path.join("relation"), Durability::D1)
        } else {
            uc_relation::RelationEngine::open(&path.join("relation"), Durability::D1)
        }
        .unwrap();
        for ((name, report), position) in [("memory", mr), ("entity", er), ("relation", rr)]
            .into_iter()
            .zip(expected)
        {
            check_report(name, &report, position);
        }
        Self {
            memory,
            entity,
            relation,
        }
    }
    fn stores(self) -> Vec<Arc<dyn Store>> {
        let entity: Arc<dyn Store> = Arc::new(EntityStore::new(self.entity));
        vec![
            Arc::new(MemoryStore::new(self.memory, entity.clone())),
            entity,
            Arc::new(RelationStore::new(self.relation)),
        ]
    }
    fn checkpoint(&self) -> [CheckpointRef; 3] {
        [
            self.memory
                .log()
                .checkpoint(uc_memory::encode_state)
                .unwrap(),
            self.entity
                .log()
                .checkpoint(uc_entity::encode_state)
                .unwrap(),
            self.relation
                .log()
                .checkpoint(uc_relation::encode_state)
                .unwrap(),
        ]
    }
}
fn check_report(name: &str, report: &OpenReport, position: Option<Position>) {
    assert_eq!(
        report.checkpoint, position,
        "{name}: checkpoint must actually be used"
    );
    assert!(
        report.rejected_checkpoints.is_empty(),
        "{name}: {:?}",
        report.rejected_checkpoints
    );
    assert_eq!(report.dropped_tail, None);
    assert!(report.gaps.is_empty());
    println!(
        "{name}: accepted checkpoint {:?}, rejected=0",
        report.checkpoint
    );
}

fn import(client: &mut Client, f: &Fixture) -> Sidecar {
    let mut sidecar = Sidecar::new();
    client.use_table("memory");
    for r in &f.memories {
        let (id, fields) = memory_fields(r).unwrap();
        client.ok(Request::Insert { id, fields });
    }
    client.use_table("entity");
    for r in &f.entities {
        let source_id = r.get("id").string();
        assert_eq!(source_id, entity_identity(r.get("name").string()));
        sidecar.entities.insert(
            source_id.into(),
            subset(r, &["kind", "created_at", "updated_at"]),
        );
        client.ok(Request::Insert {
            id: mapped_id(source_id, false).unwrap(),
            fields: f.entity_fields(r),
        });
    }
    client.use_table("relation");
    for r in &f.relations {
        let source_id = r.get("id").string();
        let key = format!(
            "{}|{}|{}",
            r.get("subject_entity_id").string(),
            normalized(r.get("relation").string()),
            r.get("object_entity_id").string()
        );
        assert_eq!(
            source_id,
            &cm_trace::hex(&cm_trace::sha256(key.as_bytes()))[..12]
        );
        sidecar
            .relations
            .insert(source_id.into(), subset(r, &["created_at", "updated_at"]));
        client.ok(Request::Insert {
            id: mapped_id(source_id, false).unwrap(),
            fields: relation_fields(r),
        });
    }
    client.use_table("memory");
    for r in &f.links {
        let (m, e) = (r.get("memory_id").string(), r.get("entity_id").string());
        assert!(
            sidecar
                .links
                .insert((m.into(), e.into()), r.get("created_at").clone())
                .is_none()
        );
        client.ok(Request::Link {
            left: mapped_id(m, true).unwrap(),
            right: mapped_id(e, false).unwrap(),
            relation: "mentions".into(),
        });
    }
    sidecar
}

// Named canonical field tuples: arrays of [field-name, value], lexicographic names.
// Memory uses ALL reconstructed original fields. Entity kind and Relation time
// originals have no exact persisted home, so are excluded from these digests.
// Their lossy projections are nevertheless checked at EVERY stage below.
#[derive(Debug, PartialEq, Eq)]
struct Digests {
    counts: [usize; 4],
    records: Vec<(String, String, String)>,
}
fn digest(kind: &str, id: String, record: Json) -> (String, String, String) {
    let tuple = Json::Array(
        record
            .object()
            .iter()
            .map(|(k, v)| Json::Array(vec![Json::Str(k.clone()), v.clone()]))
            .collect(),
    );
    (
        kind.into(),
        id,
        cm_trace::hex(&cm_trace::sha256(tuple.canonical().as_bytes())),
    )
}
fn source_digests(f: &Fixture) -> Digests {
    let mut records = Vec::new();
    for r in &f.memories {
        records.push(digest("memory", r.get("id").string().into(), r.clone()));
    }
    for r in &f.entities {
        let mut original = subset(r, &["id", "name", "aliases"]).object().clone();
        original.insert(
            "mention_count".into(),
            Json::Number(f.mentions(r.get("id").string()).to_string()),
        );
        records.push(digest(
            "entity",
            r.get("id").string().into(),
            Json::Object(original),
        ));
    }
    for r in &f.relations {
        records.push(digest(
            "relation",
            r.get("id").string().into(),
            subset(
                r,
                &["id", "subject_entity_id", "relation", "object_entity_id"],
            ),
        ));
    }
    for r in &f.links {
        records.push(digest(
            "memory_entity",
            format!(
                "{}|{}",
                r.get("memory_id").string(),
                r.get("entity_id").string()
            ),
            subset(r, &["memory_id", "entity_id"]),
        ));
    }
    records.sort();
    Digests {
        counts: [
            f.memories.len(),
            f.entities.len(),
            f.relations.len(),
            f.links.len(),
        ],
        records,
    }
}
fn verify(client: &mut Client, f: &Fixture) -> Digests {
    let mut records = Vec::new();
    let mut counts = [0; 4];
    for (i, table) in ["memory", "entity", "relation"].iter().enumerate() {
        client.use_table(table);
        let Response::Rows { rows } = client.request(Request::Query {
            select: Selection::All,
            filter: vec![],
            limit: None,
        }) else {
            panic!("query rows")
        };
        counts[i] = rows.len();
    }
    client.use_table("memory");
    for r in &f.memories {
        let (id, expected) = memory_fields(r).unwrap();
        let fields = client.get(id);
        assert_eq!(fields, expected, "all 13 projections and envelope");
        let original = reconstruct_memory(id, &fields);
        for (key, value) in r.object() {
            assert_eq!(
                original.get(key),
                value,
                "memory {} field {key}",
                r.get("id").string()
            );
        }
        assert_eq!(original_id(id, true), r.get("id").string());
        records.push(digest(
            "memory",
            original.get("id").string().into(),
            original,
        ));
    }
    client.use_table("entity");
    for r in &f.entities {
        let id = mapped_id(r.get("id").string(), false).unwrap();
        let fields = client.get(id);
        assert_eq!(fields, f.entity_fields(r));
        assert_eq!(original_id(id, false), r.get("id").string());
        records.push(digest(
            "entity",
            original_id(id, false),
            obj([
                ("id".into(), Json::Str(original_id(id, false))),
                ("name".into(), unscan(field(&fields, 0))),
                ("aliases".into(), unscan(field(&fields, 3))),
                ("mention_count".into(), unscan(field(&fields, 2))),
            ]),
        ));
    }
    client.use_table("relation");
    for r in &f.relations {
        let id = mapped_id(r.get("id").string(), false).unwrap();
        let fields = client.get(id);
        assert_eq!(fields, relation_fields(r));
        assert_eq!(original_id(id, false), r.get("id").string());
        // Query each endpoint/label separately and the complete triple, using
        // generic Query (FilterEq only advertises the subject index).
        for tags in [vec![0], vec![1], vec![2], vec![0, 1, 2]] {
            let filter = tags
                .iter()
                .map(|t| Predicate {
                    field: *t,
                    op: CompareOp::Eq,
                    value: field(&fields, *t).clone(),
                })
                .collect();
            let Response::Rows { rows } = client.request(Request::Query {
                select: Selection::All,
                filter,
                limit: None,
            }) else {
                panic!("relation query")
            };
            let mut expected: Vec<_> = f
                .relations
                .iter()
                .filter(|r| {
                    tags.iter()
                        .all(|t| field(&relation_fields(r), *t) == field(&fields, *t))
                })
                .map(|r| {
                    (
                        mapped_id(r.get("id").string(), false).unwrap(),
                        relation_fields(r),
                    )
                })
                .collect();
            expected.sort_by_key(|(id, _)| *id);
            assert_eq!(rows, expected);
        }
        records.push(digest(
            "relation",
            original_id(id, false),
            obj([
                ("id".into(), Json::Str(original_id(id, false))),
                ("subject_entity_id".into(), unscan(field(&fields, 0))),
                ("relation".into(), unscan(field(&fields, 1))),
                ("object_entity_id".into(), unscan(field(&fields, 2))),
            ]),
        ));
    }
    client.use_table("memory");
    let mut expected = Vec::new();
    for link in &f.links {
        let m = f
            .memories
            .iter()
            .find(|m| m.get("id") == link.get("memory_id"))
            .unwrap();
        let e = f
            .entities
            .iter()
            .find(|e| e.get("id") == link.get("entity_id"))
            .unwrap();
        expected.push(JoinedRow {
            left_id: mapped_id(m.get("id").string(), true).unwrap(),
            left: memory_fields(m).unwrap().1,
            right_id: mapped_id(e.get("id").string(), false).unwrap(),
            right: f.entity_fields(e),
        });
    }
    expected.sort_by_key(|r| (r.left_id, r.right_id));
    let Response::JoinedRows { rows } = client.request(Request::Join(JoinSpec {
        relation: JoinRelation::Neighbors(Some("mentions".into())),
        right_table: Some("entity".into()),
        left: Selection::All,
        right: Selection::All,
        left_filter: vec![],
        right_filter: vec![],
        limit: None,
    })) else {
        panic!("foreign Join")
    };
    assert_eq!(rows, expected);
    counts[3] = rows.len();
    assert_eq!(
        client.request(Request::CountEdges {
            relation: "mentions".into()
        }),
        Response::Count {
            count: rows.len() as u64
        }
    );
    for row in rows {
        let (m, e) = (
            original_id(row.left_id, true),
            original_id(row.right_id, false),
        );
        records.push(digest(
            "memory_entity",
            format!("{m}|{e}"),
            obj([
                ("memory_id".into(), Json::Str(m)),
                ("entity_id".into(), Json::Str(e)),
            ]),
        ));
    }
    // Reverse neighbors are requested from Memory with an Entity id, exactly
    // Step 4c's contract. Join is directed Memory -> Entity; no reverse registry edge.
    for (source, key, target, is_memory) in [
        (&f.memories, "memory_id", "entity_id", true),
        (&f.entities, "entity_id", "memory_id", false),
    ] {
        for r in source {
            let source_id = r.get("id").string();
            let id = mapped_id(source_id, is_memory).unwrap();
            let mut expected: Vec<_> = f
                .links
                .iter()
                .filter(|l| l.get(key).string() == source_id)
                .map(|l| mapped_id(l.get(target).string(), !is_memory).unwrap())
                .collect();
            expected.sort();
            for request in [
                Request::NeighborsByRelation {
                    id,
                    relation: "mentions".into(),
                },
                Request::Neighbors { id },
            ] {
                assert_eq!(
                    client.request(request),
                    Response::RecordList {
                        records: expected.clone()
                    }
                );
            }
        }
    }
    records.sort();
    Digests { counts, records }
}

// Copy only closed stores; byte maps verify no extra/missing/modified file.
fn files(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    let mut out = BTreeMap::new();
    for table in ["memory", "entity", "relation"] {
        for entry in fs::read_dir(root.join(table)).unwrap() {
            let entry = entry.unwrap();
            assert!(entry.file_type().unwrap().is_file());
            out.insert(
                PathBuf::from(table).join(entry.file_name()),
                fs::read(entry.path()).unwrap(),
            );
        }
    }
    out
}
fn backup(source: &Path, target: &Path) {
    assert_ne!(
        source.canonicalize().unwrap(),
        target.canonicalize().unwrap()
    );
    for table in ["memory", "entity", "relation"] {
        fs::create_dir(target.join(table)).unwrap();
    }
    for name in files(source).keys() {
        fs::copy(source.join(name), target.join(name)).unwrap();
    }
    assert_eq!(files(source), files(target));
}

#[test]
fn synthetic_export_socket_checkpoint_backup_and_independent_rebuild() {
    let fixture = Fixture::read();
    let source = source_digests(&fixture);
    let original = support::Temp::new();
    let restored = support::Temp::new();
    {
        let server = Server::start(Engines::open(&original.0, true, [None; 3]).stores());
        let mut client = server.client();
        let sidecar = import(&mut client, &fixture);
        assert_eq!(verify(&mut client, &fixture), source);
        sidecar.verify(&fixture); // Import-time only; consumes and destroys sidecar.
    }
    // Store wrappers own private engines. Close the listener, open engines to
    // checkpoint, then close them before verifying checkpoint acceptance over TCP.
    let checkpoints = {
        let engines = Engines::open(&original.0, false, [None; 3]);
        engines.checkpoint()
    };
    let positions = checkpoints.each_ref().map(|cp| Some(cp.position));
    {
        let server = Server::start(Engines::open(&original.0, false, positions).stores());
        assert_eq!(verify(&mut server.client(), &fixture), source);
    }
    let untouched = files(&original.0);
    backup(&original.0, &restored.0);
    {
        let server = Server::start(Engines::open(&restored.0, false, positions).stores());
        assert_eq!(verify(&mut server.client(), &fixture), source);
    }
    assert_eq!(files(&original.0), untouched);
    let before_removal = files(&restored.0);
    let mut expected_after_removal = before_removal.clone();
    for cp in &checkpoints {
        let relative = cp.path.strip_prefix(&original.0).unwrap();
        let path = restored.0.join(relative).canonicalize().unwrap();
        assert!(path.starts_with(restored.0.canonicalize().unwrap()));
        assert!(
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("checkpoint-")
        );
        assert_eq!(path.extension().unwrap(), "uc1");
        fs::remove_file(path).unwrap();
        assert!(expected_after_removal.remove(relative).is_some());
    }
    assert_eq!(
        files(&restored.0),
        expected_after_removal,
        "delete only the three actual checkpoints"
    );
    let rebuilt = {
        let server = Server::start(Engines::open(&restored.0, false, [None; 3]).stores());
        verify(&mut server.client(), &fixture)
    };
    assert_eq!(source, rebuilt);
    println!(
        "source counts={:?}; rebuilt counts={:?}",
        source.counts, rebuilt.counts
    );
    for (source, rebuilt) in source.records.iter().zip(&rebuilt.records) {
        println!(
            "{} {} source={} rebuilt={}",
            source.0, source.1, source.2, rebuilt.2
        );
    }
    assert_eq!(
        files(&original.0),
        untouched,
        "restore/rebuild must not touch original files"
    );
    assert_eq!(
        files(&restored.0),
        expected_after_removal,
        "read-only rebuild leaves log bytes intact"
    );
}

#[test]
fn fixture_covers_optional_metadata_precision_tombstone_and_case_identity() {
    let f = Fixture::read();
    assert_eq!(
        [
            f.memories.len(),
            f.entities.len(),
            f.relations.len(),
            f.links.len()
        ],
        [20, 8, 8, 20]
    );
    for key in [
        "capture_id",
        "subject",
        "predicate",
        "object",
        "superseded_by",
        "doc_id",
        "chunk_index",
        "remind_at",
        "memory_type",
        "status",
        "node_id",
        "client",
        "source_capture_id",
        "deleted_at",
    ] {
        assert!(
            f.memories.iter().any(|m| *m.get(key) == Json::Null),
            "{key}: absent"
        );
        assert!(
            f.memories.iter().any(|m| *m.get(key) != Json::Null),
            "{key}: present"
        );
    }
    for (index, variant) in [(1, "null"), (2, "array"), (3, "number")] {
        let m = &f.memories[index];
        assert!(matches!(
            (variant, m.get("metadata")),
            ("null", Json::Null) | ("array", Json::Array(_)) | ("number", Json::Number(_))
        ));
        let (id, fields) = memory_fields(m).unwrap();
        assert_eq!(reconstruct_memory(id, &fields), *m);
    }
    assert!(matches!(f.memories[0].get("metadata"), Json::Object(_)));
    assert_eq!(
        milliseconds(f.memories[0].get("created_at").string()).unwrap(),
        1788957296123
    );
    assert_eq!(
        f.memories[0].get("deleted_at").string(),
        "2026-09-09T13:00:00.987654321+00:00"
    );
    assert_eq!(f.memories[0].get("superseded_by"), f.memories[1].get("id"));
    for tag in [7, 8, 12] {
        assert_eq!(
            field(&memory_fields(&f.memories[1]).unwrap().1, tag),
            field(&memory_fields(&f.memories[3]).unwrap().1, tag)
        );
    }
    assert_ne!(
        f.memories[1].get("memory_type"),
        f.memories[3].get("memory_type")
    );
    let e = &f.entities[0];
    let aliases = e.get("aliases").array();
    assert_ne!(aliases[0], aliases[1]);
    for alias in aliases {
        assert_eq!(entity_identity(alias.string()), e.get("id").string());
    }
    assert_eq!(
        f.entities
            .iter()
            .filter(|r| r.get("id") == e.get("id"))
            .count(),
        1
    );
}

#[test]
fn projections_match_literal_frozen_field_tags() {
    use ScanValue::*;
    let f = Fixture::read();
    let (_, fields) = memory_fields(&f.memories[0]).unwrap();
    let literals = vec![
        (0, Str("Synthetic memory 0 — line one\nline two".into())),
        (1, Str("fact".into())),
        (2, StrList(vec!["synthetic".into(), "tag-0".into()])),
        (3, Str("step5-fixture".into())),
        (5, I64(1788957296123)),
        (6, I64(1788957296123)),
        (7, Str("fact".into())),
        (8, Str("active".into())),
        (9, Bool(true)),
        (10, I64(0)),
        (11, I64(1788958800987)),
        (12, Str("synthetic-node".into())),
    ];
    assert_eq!(fields.len(), 13);
    for (tag, value) in literals {
        assert_eq!(field(&fields, tag), &value);
    }
    let Str(encoded) = field(&fields, 4) else {
        panic!("metadata envelope")
    };
    let envelope = parse(encoded).unwrap();
    assert_eq!(
        envelope.get("original_metadata"),
        f.memories[0].get("metadata")
    );
    let extra = envelope.get("_remind_me_migration_extra");
    assert_eq!(
        extra.get("created_at").string(),
        "2026-09-09T12:34:56.123456789+00:00"
    );
    assert_eq!(
        extra.get("deleted_at").string(),
        "2026-09-09T13:00:00.987654321+00:00"
    );
    assert!(!envelope.object().contains_key("role"));
    assert!(!extra.object().contains_key("role"));
    assert_eq!(
        f.entity_fields(&f.entities[0]),
        vec![
            (0, Str("Ada Lovelace".into())),
            (1, Str("concept".into())),
            (2, I64(3)),
            (
                3,
                StrList(vec!["ADA LOVELACE".into(), "ada lovelace".into()])
            ),
        ]
    );
    assert_eq!(
        relation_fields(&f.relations[0]),
        vec![
            (0, Str("177f85df57ad".into())),
            (1, Str("relates to".into())),
            (2, Str("9ae82d97eba6".into())),
            (3, I64(1788957296123)),
            (4, I64(1788957356987)),
            (5, Str(String::new())),
            (6, I64(0)),
        ]
    );
}

#[test]
fn rejects_bad_memory_ids_and_timestamps_before_insert() {
    let fixture = Fixture::read();
    let source = &fixture.memories[0];
    for bad in [
        "0000000112344abc8def000000000001",
        "mem_1234",
        "mem_gg00000112344abc8def000000000001",
        "MEM_0000000112344abc8def000000000001",
    ] {
        let mut r = source.object().clone();
        r.insert("id".into(), Json::Str(bad.into()));
        assert!(memory_fields(&Json::Object(r)).is_err(), "{bad}");
    }
    for key in ["created_at", "updated_at", "deleted_at"] {
        for bad in [
            "not a timestamp",
            "2026-02-29T00:00:00Z",
            "2026-09-09T25:00:00Z",
            "2026-09-09T00:00:00",
            "2026-09-09T00:00:00. Z",
            "2026-09-09T00:00:00+24:00",
        ] {
            let mut r = source.object().clone();
            r.insert(key.into(), Json::Str(bad.into()));
            assert!(memory_fields(&Json::Object(r)).is_err(), "{key}={bad}");
        }
    }
    for (s, expected) in [
        ("1970-01-01T00:00:00Z", 0),
        ("1969-12-31T23:59:59.999999999Z", -1),
        ("1970-01-01T01:00:00+01:00", 0),
        ("1970-01-01T00:00:00-01:00", 3600000),
        ("2000-02-29T00:00:00Z", 951782400000),
    ] {
        assert_eq!(milliseconds(s).unwrap(), expected);
    }
    assert!(milliseconds("2016-12-31T23:59:60Z").is_err());
    assert!(mapped_id("123456789abc00", false).is_err());
    assert!(mapped_id("123456789abg", false).is_err());
    assert_ne!(
        mapped_id("123456789abc", false).unwrap(),
        mapped_id("123456789abd", false).unwrap()
    );
    let id = mapped_id("mem_00112233445546778899aabbccddeeff", true).unwrap();
    assert_eq!(
        id.0,
        [
            0, 17, 34, 51, 68, 85, 70, 119, 136, 153, 170, 187, 204, 221, 238, 255
        ]
    );
    assert_eq!(
        original_id(id, true),
        "mem_00112233445546778899aabbccddeeff"
    );
}

#[test]
fn json_parser_canonical_roundtrip_and_rejections() {
    for s in [
        r#"{"z":null,"a":[true,false,-2,0.05,1e-3,"\"\\\n\u4e8c\ud83d\ude00"]}"#,
        "42",
        "null",
        r#""scalar""#,
    ] {
        let j = parse(s).unwrap();
        assert_eq!(parse(&j.canonical()).unwrap(), j);
    }
    assert_eq!(parse(r#""\ud83d\ude00""#).unwrap(), Json::Str("😀".into()));
    for s in [
        "[1,]",
        "{\"a\":1,\"a\":2}",
        "01",
        "1.",
        "1e",
        "true false",
        r#""\ud800""#,
        r#""\udc00""#,
        r#""\q""#,
        "\"\n\"",
        "[",
    ] {
        assert!(parse(s).is_err(), "{s}");
    }
    assert_eq!(
        cm_trace::hex(&cm_trace::sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
