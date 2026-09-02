//! R27's closed-scope, immutable v2 reference catalog and transactional mapper.

use crate::mapping::{MappingError, map_validated_core};
use exp1_record_format::Record;
use exp1_workload_conformance::{
    Error as SemanticError, MANIFEST_DIGEST_PROFILE_V2, MANIFEST_DOMAIN_V2, Segment,
    ValidationContextV2, hex, manifest_digest_v2, sha256, validate_manifest_v2,
    validate_semantic_operation_v2, validate_stream_v2, workload_digest_v2,
};
use std::collections::{BTreeMap, BTreeSet};

const SCOPE_PROFILE: &str = "EXP-0001-R23-CLOSED-STREAM-SCOPE-JCS-v2";
const SCOPE_DIGEST_PROFILE: &str = "EXP-0001-R23-CLOSED-STREAM-SCOPE-DIGEST-v2";
const SCOPE_DOMAIN: &str = "rusty-data-os/exp1/closed-stream-scope/v2";

pub struct ScopeDigestInput<'a> {
    pub descriptor: &'a [u8],
    pub artifact_metadata: &'a [u8],
}
pub struct ManifestBindingInput<'a> {
    pub manifest: &'a [u8],
    pub manifest_digest_descriptor: &'a [u8],
    pub manifest_artifact_metadata: &'a [u8],
    pub stream: &'a [u8],
    pub stream_artifact_metadata: &'a [u8],
}
pub struct ClosedScopeInputV2<'a> {
    pub scope: ScopeDigestInput<'a>,
    pub members: &'a [ManifestBindingInput<'a>],
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextConstructionError {
    InvalidScopeEncoding,
    UnsupportedScopeProfile,
    InvalidScopeDigest,
    ScopeReferenceFailure,
    InvalidCellAuthority,
    InvalidMemberBinding,
    OmittedStream,
    ExtraStream,
    DuplicateStreamNamespace,
    SubstitutedStream,
    ForeignWorkloadOrCell,
    SemanticValidation(SemanticError),
    IdentityCollision,
    ResourceLimit,
    Extraction,
    SelectedStreamMissing,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
enum Role {
    Request,
    Event,
    Information,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FactClass {
    Ordinary,
    Correction,
    Retraction,
}
#[derive(Clone, Debug)]
struct IdentityBinding {
    id: [u8; 16],
    role: Role,
    namespace: [u8; 16],
    total_position: u64,
    segment: Segment,
    ordinal: u64,
    producer: [u8; 16],
    producer_ordinal: u64,
    fact: Option<FactClass>,
}
#[derive(Clone, Debug)]
struct Operation {
    bytes: Vec<u8>,
    namespace: [u8; 16],
    segment: Segment,
    ordinal: u64,
    total_position: u64,
    producer: [u8; 16],
    producer_ordinal: u64,
    request: [u8; 16],
    event: [u8; 16],
    information: [u8; 16],
    references: Vec<[u8; 16]>,
}

#[derive(Debug)]
pub struct ReferenceCatalogV2 {
    scope_id: [u8; 16],
    cell_id: String,
    selected: [u8; 16],
    stream_count: usize,
    source_bytes: usize,
    operation_count: usize,
    identities: BTreeMap<[u8; 16], IdentityBinding>,
    selected_operations: Vec<Operation>,
}
impl ReferenceCatalogV2 {
    pub fn scope_id(&self) -> [u8; 16] {
        self.scope_id
    }
    pub fn cell_id(&self) -> &str {
        &self.cell_id
    }
    pub fn selected_stream_namespace(&self) -> [u8; 16] {
        self.selected
    }
    pub fn stream_count(&self) -> usize {
        self.stream_count
    }
    pub fn source_bytes(&self) -> usize {
        self.source_bytes
    }
    pub fn operation_count(&self) -> usize {
        self.operation_count
    }
    pub fn identity_entry_count(&self) -> usize {
        self.identities.len()
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AcceptedPrefixStateV2 {
    accepted: u64,
    sequence: u64,
    ordinal: u64,
    scope_id: [u8; 16],
    namespace: [u8; 16],
}
impl AcceptedPrefixStateV2 {
    pub fn accepted_operations(&self) -> u64 {
        self.accepted
    }
    pub fn previous_sequence(&self) -> u64 {
        self.sequence
    }
    pub fn previous_physical_ordinal(&self) -> u64 {
        self.ordinal
    }
}
#[derive(Debug)]
pub struct ReferenceContextV2 {
    catalog: ReferenceCatalogV2,
    initial: AcceptedPrefixStateV2,
}
impl ReferenceContextV2 {
    pub fn catalog(&self) -> &ReferenceCatalogV2 {
        &self.catalog
    }
    pub fn initial_state(&self) -> &AcceptedPrefixStateV2 {
        &self.initial
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReferenceError {
    Missing,
    Future,
    WrongKind,
    WrongFact,
    SelfReference,
    CrossStream,
    CrossSegment,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContextualMappingError {
    SemanticValidation(SemanticError),
    Discontinuity,
    Exhaustion,
    Reference(ReferenceError),
    Mapping(MappingError),
    ResourceLimit,
    Extraction,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContextualMappedRecordV2 {
    frame: Vec<u8>,
    record: Record,
    next: AcceptedPrefixStateV2,
}
impl ContextualMappedRecordV2 {
    pub fn frame(&self) -> &[u8] {
        &self.frame
    }
    pub fn record(&self) -> &Record {
        &self.record
    }
    pub fn next_state(&self) -> &AcceptedPrefixStateV2 {
        &self.next
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum J {
    S(String),
    A(Vec<J>),
    O(BTreeMap<String, J>),
}
struct P<'a> {
    b: &'a [u8],
    p: usize,
}
impl P<'_> {
    fn value(&mut self) -> Result<J, ()> {
        match self.b.get(self.p) {
            Some(b'"') => self.string().map(J::S),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            _ => Err(()),
        }
    }
    fn string(&mut self) -> Result<String, ()> {
        self.p += 1;
        let mut s = String::new();
        loop {
            let c = *self.b.get(self.p).ok_or(())?;
            self.p += 1;
            match c {
                b'"' => return Ok(s),
                0..=31 => return Err(()),
                b'\\' => {
                    let e = *self.b.get(self.p).ok_or(())?;
                    self.p += 1;
                    match e {
                        b'"' => s.push('"'),
                        b'\\' => s.push('\\'),
                        b'/' => s.push('/'),
                        b'b' => s.push('\x08'),
                        b'f' => s.push('\x0c'),
                        b'n' => s.push('\n'),
                        b'r' => s.push('\r'),
                        b't' => s.push('\t'),
                        b'u' => {
                            let h = std::str::from_utf8(self.b.get(self.p..self.p + 4).ok_or(())?)
                                .map_err(|_| ())?;
                            self.p += 4;
                            let n = u16::from_str_radix(h, 16).map_err(|_| ())?;
                            let cp = if (0xd800..=0xdbff).contains(&n) {
                                if self.b.get(self.p..self.p + 2) != Some(b"\\u") {
                                    return Err(());
                                }
                                self.p += 2;
                                let h =
                                    std::str::from_utf8(self.b.get(self.p..self.p + 4).ok_or(())?)
                                        .map_err(|_| ())?;
                                self.p += 4;
                                let low = u16::from_str_radix(h, 16).map_err(|_| ())?;
                                if !(0xdc00..=0xdfff).contains(&low) {
                                    return Err(());
                                }
                                0x10000 + (((n as u32 - 0xd800) << 10) | (low as u32 - 0xdc00))
                            } else if (0xdc00..=0xdfff).contains(&n) {
                                return Err(());
                            } else {
                                n as u32
                            };
                            s.push(char::from_u32(cp).ok_or(())?)
                        }
                        _ => return Err(()),
                    }
                }
                0x20..=0x7f => s.push(c as char),
                _ => {
                    self.p -= 1;
                    let x = std::str::from_utf8(&self.b[self.p..])
                        .map_err(|_| ())?
                        .chars()
                        .next()
                        .ok_or(())?;
                    s.push(x);
                    self.p += x.len_utf8()
                }
            }
        }
    }
    fn object(&mut self) -> Result<J, ()> {
        self.p += 1;
        let mut m = BTreeMap::new();
        if self.b.get(self.p) == Some(&b'}') {
            self.p += 1;
            return Ok(J::O(m));
        }
        loop {
            if self.b.get(self.p) != Some(&b'"') {
                return Err(());
            }
            let k = self.string()?;
            if self.b.get(self.p) != Some(&b':') {
                return Err(());
            }
            self.p += 1;
            let v = self.value()?;
            if m.insert(k, v).is_some() {
                return Err(());
            }
            match self.b.get(self.p) {
                Some(b',') => self.p += 1,
                Some(b'}') => {
                    self.p += 1;
                    break;
                }
                _ => return Err(()),
            }
        }
        Ok(J::O(m))
    }
    fn array(&mut self) -> Result<J, ()> {
        self.p += 1;
        let mut a = vec![];
        if self.b.get(self.p) == Some(&b']') {
            self.p += 1;
            return Ok(J::A(a));
        }
        loop {
            a.push(self.value()?);
            match self.b.get(self.p) {
                Some(b',') => self.p += 1,
                Some(b']') => {
                    self.p += 1;
                    break;
                }
                _ => return Err(()),
            }
        }
        Ok(J::A(a))
    }
}
fn quote(s: &str, o: &mut String) {
    o.push('"');
    for c in s.chars() {
        match c {
            '"' => o.push_str("\\\""),
            '\\' => o.push_str("\\\\"),
            '\x08' => o.push_str("\\b"),
            '\x0c' => o.push_str("\\f"),
            '\n' => o.push_str("\\n"),
            '\r' => o.push_str("\\r"),
            '\t' => o.push_str("\\t"),
            c if c < ' ' => o.push_str(&format!("\\u{:04x}", c as u32)),
            c => o.push(c),
        }
    }
    o.push('"')
}
fn canon(j: &J, o: &mut String) {
    match j {
        J::S(s) => quote(s, o),
        J::A(a) => {
            o.push('[');
            for (i, v) in a.iter().enumerate() {
                if i > 0 {
                    o.push(',')
                }
                canon(v, o)
            }
            o.push(']')
        }
        J::O(m) => {
            o.push('{');
            let mut entries: Vec<_> = m.iter().collect();
            entries.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
            for (i, (k, v)) in entries.into_iter().enumerate() {
                if i > 0 {
                    o.push(',')
                }
                quote(k, o);
                o.push(':');
                canon(v, o)
            }
            o.push('}')
        }
    }
}
fn parse(b: &[u8]) -> Result<J, ()> {
    std::str::from_utf8(b).map_err(|_| ())?;
    let mut p = P { b, p: 0 };
    let j = p.value()?;
    if p.p != b.len() {
        return Err(());
    }
    let mut s = String::new();
    canon(&j, &mut s);
    if s.as_bytes() != b {
        return Err(());
    }
    Ok(j)
}
fn obj(j: &J) -> Result<&BTreeMap<String, J>, ()> {
    if let J::O(x) = j { Ok(x) } else { Err(()) }
}
fn arr(j: &J) -> Result<&[J], ()> {
    if let J::A(x) = j { Ok(x) } else { Err(()) }
}
fn strv(j: &J) -> Result<&str, ()> {
    if let J::S(x) = j { Ok(x) } else { Err(()) }
}
fn exact(o: &BTreeMap<String, J>, keys: &[&str]) -> Result<(), ()> {
    if o.len() != keys.len() || keys.iter().any(|k| !o.contains_key(*k)) {
        Err(())
    } else {
        Ok(())
    }
}
fn uuid(s: &str) -> Result<[u8; 16], ()> {
    if s.len() != 36
        || s.as_bytes()[8] != b'-'
        || s.as_bytes()[13] != b'-'
        || s.as_bytes()[18] != b'-'
        || s.as_bytes()[23] != b'-'
    {
        return Err(());
    }
    let h: String = s.chars().filter(|&c| c != '-').collect();
    if h.len() != 32
        || !h
            .bytes()
            .all(|c| c.is_ascii_digit() || matches!(c, b'a'..=b'f'))
    {
        return Err(());
    }
    let mut v = [0; 16];
    for (i, x) in v.iter_mut().enumerate() {
        *x = u8::from_str_radix(&h[i * 2..i * 2 + 2], 16).map_err(|_| ())?
    }
    if v == [0; 16] || v[8] & 0xc0 != 0x80 {
        return Err(());
    }
    Ok(v)
}
fn sha(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|c| c.is_ascii_digit() || matches!(c, b'a'..=b'f'))
}
fn dec(s: &str) -> Result<u64, ()> {
    if s.is_empty() || (s.len() > 1 && s.starts_with('0')) || !s.bytes().all(|x| x.is_ascii_digit())
    {
        return Err(());
    }
    s.parse().map_err(|_| ())
}
fn fields<'a>(b: &'a [u8], magic: &[u8], n: u16) -> Option<Vec<&'a [u8]>> {
    if b.get(..magic.len())? != magic
        || u16::from_be_bytes(b.get(magic.len()..magic.len() + 2)?.try_into().ok()?) != n
    {
        return None;
    }
    let mut p = magic.len() + 2;
    let mut v = Vec::with_capacity(n as usize);
    for tag in 1..=n {
        if *b.get(p)? != tag as u8 {
            return None;
        }
        let z = u32::from_be_bytes(b.get(p + 1..p + 5)?.try_into().ok()?) as usize;
        p += 5;
        let e = p.checked_add(z)?;
        v.push(b.get(p..e)?);
        p = e
    }
    (p == b.len()).then_some(v)
}
fn hex_digest(domain: &str, b: &[u8]) -> String {
    let mut p = domain.as_bytes().to_vec();
    p.push(0);
    p.extend(b);
    hex(&sha256(&p))
}

fn required_string<'a>(o: &'a BTreeMap<String, J>, key: &str) -> Result<&'a str, ()> {
    strv(o.get(key).ok_or(())?)
}

fn exact_state(o: &BTreeMap<String, J>, state: &str) -> Result<(), ()> {
    exact(o, &["state"])?;
    (required_string(o, "state")? == state)
        .then_some(())
        .ok_or(())
}

/// Resolve one immutable, published R7 artifact entry.  This deliberately validates the
/// complete closed R7 shapes used here rather than treating metadata as an opaque digest bag.
fn r7_artifact(metadata: &[u8], artifact_id: &str) -> Result<BTreeMap<String, J>, ()> {
    let root = parse(metadata)?;
    let root = obj(&root)?;
    exact(
        root,
        &[
            "body",
            "correction_reason",
            "created_at_utc_ns",
            "record_id",
            "record_kind",
            "run_id",
            "schema_version",
            "series_id",
            "supersedes_record_id",
        ],
    )?;
    if required_string(root, "record_kind")? != "artifact_manifest"
        || required_string(root, "schema_version")? != "EXP1-R7-JSON-JCS-1"
        || uuid(required_string(root, "record_id")?).is_err()
        || uuid(required_string(root, "series_id")?).is_err()
        || dec(required_string(root, "created_at_utc_ns")?).is_err()
    {
        return Err(());
    }
    exact_state(
        obj(root.get("correction_reason").ok_or(())?)?,
        "not_applicable",
    )?;
    exact_state(
        obj(root.get("supersedes_record_id").ok_or(())?)?,
        "not_applicable",
    )?;
    let run = obj(root.get("run_id").ok_or(())?)?;
    exact(run, &["state", "value"])?;
    if required_string(run, "state")? != "present" || uuid(required_string(run, "value")?).is_err()
    {
        return Err(());
    }
    let body = obj(root.get("body").ok_or(())?)?;
    exact(
        body,
        &[
            "artifacts",
            "provenance_edges",
            "publication_state",
            "scope",
            "series_freeze",
        ],
    )?;
    if required_string(body, "publication_state")? != "published"
        || required_string(body, "scope")? != "run"
    {
        return Err(());
    }
    exact_state(obj(body.get("series_freeze").ok_or(())?)?, "not_applicable")?;
    let edges = arr(body.get("provenance_edges").ok_or(())?)?;
    let mut previous_edge: Option<(&str, &str, &str)> = None;
    for edge in edges {
        let edge = obj(edge)?;
        exact(edge, &["from_artifact_id", "relation", "to_artifact_id"])?;
        let from = required_string(edge, "from_artifact_id")?;
        let relation = required_string(edge, "relation")?;
        let to = required_string(edge, "to_artifact_id")?;
        if uuid(from).is_err()
            || uuid(to).is_err()
            || !matches!(
                relation,
                "generated-by"
                    | "captured-from"
                    | "configured-by"
                    | "derived-from"
                    | "validated-by"
                    | "supersedes"
                    | "generated_from"
            )
            || previous_edge.is_some_and(|previous| previous >= (from, relation, to))
        {
            return Err(());
        }
        previous_edge = Some((from, relation, to));
    }
    let artifacts = arr(body.get("artifacts").ok_or(())?)?;
    let mut found = None;
    for value in artifacts {
        let entry = obj(value)?;
        exact(
            entry,
            &[
                "artifact_id",
                "byte_length",
                "created_by_record_id",
                "logical_path",
                "media_type",
                "retention_state",
                "role",
                "sensitivity",
                "sha256",
                "uri",
                "validation_report_ids",
            ],
        )?;
        if uuid(required_string(entry, "artifact_id")?).is_err()
            || uuid(required_string(entry, "created_by_record_id")?).is_err()
            || dec(required_string(entry, "byte_length")?).is_err()
            || required_string(entry, "logical_path")?.is_empty()
            || required_string(entry, "media_type")?.is_empty()
            || required_string(entry, "sensitivity")? != "public"
            || !sha(required_string(entry, "sha256")?)
            || !matches!(
                required_string(entry, "uri")?,
                uri if uri.starts_with("https:") || uri.starts_with("file:")
            )
            || !arr(entry.get("validation_report_ids").ok_or(())?)?.is_empty()
        {
            return Err(());
        }
        if required_string(entry, "artifact_id")? == artifact_id {
            if found.is_some() || required_string(entry, "retention_state")? != "published" {
                return Err(());
            }
            found = Some(entry.clone());
        }
    }
    found.ok_or(())
}

fn ref_matches(
    reference: &BTreeMap<String, J>,
    entry: &BTreeMap<String, J>,
    bytes: &[u8],
) -> Result<(), ()> {
    exact(reference, &["artifact_id", "byte_length", "sha256", "uri"])?;
    for key in ["artifact_id", "byte_length", "sha256", "uri"] {
        if required_string(reference, key)? != required_string(entry, key)? {
            return Err(());
        }
    }
    if dec(required_string(reference, "byte_length")?)? != bytes.len() as u64
        || required_string(reference, "sha256")? != hex(&sha256(bytes))
    {
        return Err(());
    }
    Ok(())
}

struct Member {
    ns: [u8; 16],
    workload: String,
    manifest: String,
    md: String,
    sd: String,
    len: u64,
    raw: String,
}
fn scope_descriptor(
    bytes: &[u8],
) -> Result<([u8; 16], String, Vec<Member>), ContextConstructionError> {
    if bytes.len() > 262_144 {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let root = parse(bytes).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    let o = obj(&root).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    exact(
        o,
        &[
            "schema_version",
            "record_kind",
            "scope_id",
            "cell_id",
            "members",
        ],
    )
    .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    if strv(&o["schema_version"]).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
        != SCOPE_PROFILE
    {
        return Err(ContextConstructionError::UnsupportedScopeProfile);
    }
    if strv(&o["record_kind"]).ok() != Some("closed_stream_scope") {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    let id =
        uuid(strv(&o["scope_id"]).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?)
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    let cell = strv(&o["cell_id"]).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    if cell.is_empty() || cell.len() > 128 || !cell.is_ascii() {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    let a = arr(&o["members"]).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
    if a.is_empty() {
        return Err(ContextConstructionError::InvalidScopeEncoding);
    }
    if a.len() > 256 {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let mut out = vec![];
    let mut prev = None;
    let mut namespaces = BTreeSet::new();
    let mut workloads = BTreeSet::new();
    let mut manifests = BTreeSet::new();
    for j in a {
        let x = obj(j).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        exact(
            x,
            &[
                "stream_namespace",
                "workload_id",
                "manifest_id",
                "manifest_digest",
                "stream_digest",
                "stream_byte_length",
                "stream_artifact_sha256",
            ],
        )
        .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        let ns = uuid(
            strv(&x["stream_namespace"])
                .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?,
        )
        .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        let w = strv(&x["workload_id"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
            .to_owned();
        let m = strv(&x["manifest_id"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
            .to_owned();
        uuid(&w).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        uuid(&m).map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        let md = strv(&x["manifest_digest"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
            .to_owned();
        let sd = strv(&x["stream_digest"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
            .to_owned();
        let raw = strv(&x["stream_artifact_sha256"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?
            .to_owned();
        if !sha(&md) || !sha(&sd) || !sha(&raw) {
            return Err(ContextConstructionError::InvalidScopeEncoding);
        }
        let len = dec(strv(&x["stream_byte_length"])
            .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?)
        .map_err(|_| ContextConstructionError::InvalidScopeEncoding)?;
        if prev.is_some_and(|p| p >= ns)
            || !namespaces.insert(ns)
            || !workloads.insert(w.clone())
            || !manifests.insert(m.clone())
        {
            return Err(ContextConstructionError::InvalidScopeEncoding);
        }
        prev = Some(ns);
        out.push(Member {
            ns,
            workload: w,
            manifest: m,
            md,
            sd,
            len,
            raw,
        })
    }
    Ok((id, cell.to_owned(), out))
}

pub fn construct_reference_context_v2(
    input: ClosedScopeInputV2<'_>,
    selected_stream_namespace: [u8; 16],
) -> Result<ReferenceContextV2, ContextConstructionError> {
    let metadata = input
        .scope
        .artifact_metadata
        .len()
        .checked_add(
            input
                .members
                .iter()
                .try_fold(0usize, |a, m| {
                    a.checked_add(m.manifest_artifact_metadata.len())?
                        .checked_add(m.stream_artifact_metadata.len())
                })
                .ok_or(ContextConstructionError::ResourceLimit)?,
        )
        .ok_or(ContextConstructionError::ResourceLimit)?;
    if metadata > 4_194_304 {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let (scope_id, cell, members) = scope_descriptor(input.scope.descriptor)?;
    let d = parse(input.scope.artifact_metadata)
        .map_err(|_| ContextConstructionError::InvalidScopeDigest)?;
    let o = obj(&d).map_err(|_| ContextConstructionError::InvalidScopeDigest)?;
    exact(o, &["algorithm", "domain", "profile", "scope_ref", "value"])
        .map_err(|_| ContextConstructionError::InvalidScopeDigest)?;
    if strv(&o["algorithm"]).ok() != Some("SHA-256/FIPS-180-4")
        || strv(&o["domain"]).ok() != Some(SCOPE_DOMAIN)
        || strv(&o["profile"]).ok() != Some(SCOPE_DIGEST_PROFILE)
    {
        return Err(ContextConstructionError::UnsupportedScopeProfile);
    }
    if strv(&o["value"]).ok() != Some(hex_digest(SCOPE_DOMAIN, input.scope.descriptor).as_str()) {
        return Err(ContextConstructionError::InvalidScopeDigest);
    }
    let scope_ref =
        obj(&o["scope_ref"]).map_err(|_| ContextConstructionError::ScopeReferenceFailure)?;
    exact(scope_ref, &["artifact_id", "byte_length", "sha256", "uri"])
        .map_err(|_| ContextConstructionError::ScopeReferenceFailure)?;
    if uuid(
        required_string(scope_ref, "artifact_id")
            .map_err(|_| ContextConstructionError::ScopeReferenceFailure)?,
    )
    .is_err()
        || dec(required_string(scope_ref, "byte_length")
            .map_err(|_| ContextConstructionError::ScopeReferenceFailure)?)
        .ok()
            != Some(input.scope.descriptor.len() as u64)
        || required_string(scope_ref, "sha256").ok()
            != Some(hex(&sha256(input.scope.descriptor)).as_str())
        || required_string(scope_ref, "uri").map_or(true, |u| {
            !(u.starts_with("https:") || u.starts_with("file:"))
        })
    {
        return Err(ContextConstructionError::ScopeReferenceFailure);
    }
    // R27 authorizes this reviewed R8 cell only; a syntactically plausible caller label is not authority.
    if cell != "PC-D1-raw-v2" {
        return Err(ContextConstructionError::InvalidCellAuthority);
    }
    let manifest_bytes = input
        .members
        .iter()
        .try_fold(0usize, |a, m| a.checked_add(m.manifest.len()))
        .ok_or(ContextConstructionError::ResourceLimit)?;
    let stream_bytes = input
        .members
        .iter()
        .try_fold(0usize, |a, m| a.checked_add(m.stream.len()))
        .ok_or(ContextConstructionError::ResourceLimit)?;
    if manifest_bytes > 1_048_576 || stream_bytes > 16_777_216 {
        return Err(ContextConstructionError::ResourceLimit);
    }
    let mut supplied = Vec::new();
    for bind in input.members {
        let mo =
            parse(bind.manifest).map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let mt = obj(&mo).map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let g = obj(mt
            .get("generator_inputs")
            .ok_or(ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let ns = uuid(
            strv(
                g.get("stream_namespace")
                    .ok_or(ContextConstructionError::InvalidMemberBinding)?,
            )
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let workload = strv(
            mt.get("workload_id")
                .ok_or(ContextConstructionError::InvalidMemberBinding)?,
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?
        .to_owned();
        let manifest = strv(
            mt.get("manifest_id")
                .ok_or(ContextConstructionError::InvalidMemberBinding)?,
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?
        .to_owned();
        supplied.push((ns, bind, workload, manifest));
    }
    supplied.sort_by_key(|x| x.0);
    // Do not publish identities while resolving members.  In particular, two
    // otherwise-valid bindings for the same namespace necessarily contain the
    // same identities; R27 requires that condition to be classified as a
    // duplicate supplied member, not as an identity collision.  Retain only
    // the validated/extracted operations until supplied-set classification is
    // complete.
    let mut resolved_operations = Vec::with_capacity(supplied.len());
    let mut operation_count = 0usize;
    for (ns, bind, _workload, _manifest) in &supplied {
        let expected = members.iter().find(|m| m.ns == *ns);
        // Resolve the manifest descriptor and both independent R7 artifact records before set equality.
        let descriptor = parse(bind.manifest_digest_descriptor)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let dd = obj(&descriptor).map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        exact(
            dd,
            &["algorithm", "domain", "manifest_ref", "profile", "value"],
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        if required_string(dd, "algorithm").ok() != Some("SHA-256/FIPS-180-4")
            || required_string(dd, "profile").ok() != Some(MANIFEST_DIGEST_PROFILE_V2)
            || required_string(dd, "domain").ok() != Some(MANIFEST_DOMAIN_V2)
        {
            return Err(ContextConstructionError::UnsupportedScopeProfile);
        }
        let manifest_ref = obj(dd
            .get("manifest_ref")
            .ok_or(ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let manifest_artifact_id = required_string(manifest_ref, "artifact_id")
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let manifest_entry = r7_artifact(bind.manifest_artifact_metadata, manifest_artifact_id)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        ref_matches(manifest_ref, &manifest_entry, bind.manifest)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        if required_string(&manifest_entry, "role").ok() != Some("workload_manifest")
            || required_string(&manifest_entry, "media_type").ok()
                != Some("application/vnd.rusty-data-os.exp1-workload-manifest+jcs")
        {
            return Err(ContextConstructionError::InvalidMemberBinding);
        }
        let mt =
            obj(&parse(bind.manifest)
                .map_err(|_| ContextConstructionError::InvalidMemberBinding)?)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?
            .clone();
        let stream_ref = obj(mt
            .get("stream_ref")
            .ok_or(ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let stream_artifact_id = required_string(stream_ref, "artifact_id")
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let stream_entry = r7_artifact(bind.stream_artifact_metadata, stream_artifact_id)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        if required_string(&stream_entry, "role").ok() != Some("configuration")
            || required_string(&stream_entry, "media_type").ok()
                != Some("application/vnd.rusty-data-os.exp1-workload-stream")
            || required_string(&stream_entry, "byte_length")
                .and_then(dec)
                .ok()
                != Some(bind.stream.len() as u64)
            || required_string(&stream_entry, "sha256").ok()
                != Some(hex(&sha256(bind.stream)).as_str())
        {
            return Err(ContextConstructionError::InvalidMemberBinding);
        }
        if required_string(dd, "value").ok()
            != Some(hex(&manifest_digest_v2(bind.manifest)).as_str())
        {
            return Err(ContextConstructionError::InvalidMemberBinding);
        }
        let Some(expected) = expected else { continue };
        if expected.md != hex(&manifest_digest_v2(bind.manifest))
            || expected.sd != hex(&workload_digest_v2(bind.stream))
            || expected.len != bind.stream.len() as u64
            || expected.raw != hex(&sha256(bind.stream))
        {
            return Err(ContextConstructionError::InvalidMemberBinding);
        }
        let policy = obj(obj(mt
            .get("generator_inputs")
            .ok_or(ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?
        .get("reference_cardinality_policy")
        .ok_or(ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let warm = dec(required_string(
            obj(policy
                .get("warm_up")
                .ok_or(ContextConstructionError::InvalidMemberBinding)?)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
            "subsequent",
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let measured = dec(required_string(
            obj(policy
                .get("measured")
                .ok_or(ContextConstructionError::InvalidMemberBinding)?)
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
            "subsequent",
        )
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?)
        .map_err(|_| ContextConstructionError::InvalidMemberBinding)?;
        let ctx = ValidationContextV2 {
            stream: bind.stream,
            warm_up_subsequent: warm,
            measured_subsequent: measured,
            manifest_artifact_sha256: &hex(&sha256(bind.manifest)),
            manifest_artifact_length: bind.manifest.len() as u64,
            descriptor_profile: strv(
                dd.get("profile")
                    .ok_or(ContextConstructionError::InvalidMemberBinding)?,
            )
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
            descriptor_domain: strv(
                dd.get("domain")
                    .ok_or(ContextConstructionError::InvalidMemberBinding)?,
            )
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
            descriptor_value: strv(
                dd.get("value")
                    .ok_or(ContextConstructionError::InvalidMemberBinding)?,
            )
            .map_err(|_| ContextConstructionError::InvalidMemberBinding)?,
        };
        if ctx.descriptor_profile != MANIFEST_DIGEST_PROFILE_V2
            || ctx.descriptor_domain != MANIFEST_DOMAIN_V2
        {
            return Err(ContextConstructionError::UnsupportedScopeProfile);
        }
        validate_manifest_v2(bind.manifest, &ctx)
            .map_err(ContextConstructionError::SemanticValidation)?;
        let (n, _, _) = validate_stream_v2(bind.stream, warm, measured)
            .map_err(ContextConstructionError::SemanticValidation)?;
        operation_count = operation_count
            .checked_add(n as usize)
            .ok_or(ContextConstructionError::ResourceLimit)?;
        if operation_count > 65_536 {
            return Err(ContextConstructionError::ResourceLimit);
        }
        let operations =
            stream_operations(bind.stream).ok_or(ContextConstructionError::Extraction)?;
        // Classification is part of semantic extraction and must also finish
        // before supplied-set errors are considered.
        for operation in &operations {
            classify(operation)?;
        }
        resolved_operations.push((*ns, operations));
    }
    // Exact supplied-set classification belongs after complete resolution and semantic validation.
    if supplied.windows(2).any(|x| x[0].0 == x[1].0) {
        return Err(ContextConstructionError::DuplicateStreamNamespace);
    }
    if supplied.len() < members.len() {
        return Err(ContextConstructionError::OmittedStream);
    }
    if supplied.len() > members.len() {
        return Err(ContextConstructionError::ExtraStream);
    }
    for (expected, actual) in members.iter().zip(&supplied) {
        if expected.ns != actual.0 {
            return Err(ContextConstructionError::SubstitutedStream);
        }
        if expected.workload != actual.2 || expected.manifest != actual.3 {
            return Err(ContextConstructionError::ForeignWorkloadOrCell);
        }
    }
    if !members.iter().any(|m| m.ns == selected_stream_namespace) {
        return Err(ContextConstructionError::SelectedStreamMissing);
    }
    // Only an exactly classified supplied set may contribute to the global
    // typed identity catalog.  This is deliberately after selected-namespace
    // enforcement, as prescribed by R27 section 5.
    let mut identities = BTreeMap::new();
    let mut selected_ops = vec![];
    for (ns, operations) in resolved_operations {
        for op in operations {
            for (role, id, fact) in [
                (Role::Request, op.request, None),
                (Role::Event, op.event, Some(classify(&op)?)),
                (Role::Information, op.information, None),
            ] {
                if identities
                    .insert(
                        id,
                        IdentityBinding {
                            id,
                            role,
                            namespace: op.namespace,
                            total_position: op.total_position,
                            segment: op.segment,
                            ordinal: op.ordinal,
                            producer: op.producer,
                            producer_ordinal: op.producer_ordinal,
                            fact,
                        },
                    )
                    .is_some()
                {
                    return Err(ContextConstructionError::IdentityCollision);
                }
            }
            if ns == selected_stream_namespace {
                selected_ops.push(op);
            }
        }
    }
    let initial = AcceptedPrefixStateV2 {
        accepted: 0,
        sequence: 0,
        ordinal: 0,
        scope_id,
        namespace: selected_stream_namespace,
    };
    Ok(ReferenceContextV2 {
        catalog: ReferenceCatalogV2 {
            scope_id,
            cell_id: cell,
            selected: selected_stream_namespace,
            stream_count: members.len(),
            source_bytes: stream_bytes,
            operation_count,
            identities,
            selected_operations: selected_ops,
        },
        initial,
    })
}
fn stream_operations(b: &[u8]) -> Option<Vec<Operation>> {
    let h = b"RDOS-WS2EXP-0001-SEMANTIC-OP-v2";
    let n = u64::from_be_bytes(b.get(h.len()..h.len() + 8)?.try_into().ok()?);
    let mut p = h.len() + 24;
    let mut out = vec![];
    for total_position in 0..n {
        let z = usize::try_from(u64::from_be_bytes(b.get(p..p + 8)?.try_into().ok()?)).ok()?;
        p += 8;
        let e = p.checked_add(z)?;
        let mut operation = extract(b.get(p..e)?)?;
        operation.total_position = total_position;
        out.push(operation);
        p = e
    }
    Some(out)
}
fn extract(b: &[u8]) -> Option<Operation> {
    let d = validate_semantic_operation_v2(b).ok()?;
    let s = fields(b, b"RDOS-SOP2", 13)?;
    let op = fields(s[0], b"RDOS-OP1", 14)?;
    Some(Operation {
        bytes: b.to_vec(),
        namespace: d.namespace,
        segment: d.segment,
        ordinal: d.ordinal,
        total_position: 0,
        producer: op[11].try_into().ok()?,
        producer_ordinal: u64::from_be_bytes(op[12].try_into().ok()?),
        request: s[4].try_into().ok()?,
        event: s[5].try_into().ok()?,
        information: s[6].try_into().ok()?,
        references: d.references,
    })
}
fn classify(op: &Operation) -> Result<FactClass, ContextConstructionError> {
    let s = fields(&op.bytes, b"RDOS-SOP2", 13).ok_or(ContextConstructionError::Extraction)?;
    let e = fields(s[8], b"RDOS-ENV2", 13).ok_or(ContextConstructionError::Extraction)?;
    let fact = std::str::from_utf8(e[2]).map_err(|_| ContextConstructionError::Extraction)?;
    Ok(if fact.starts_with("correction") {
        FactClass::Correction
    } else if fact.starts_with("retraction") {
        FactClass::Retraction
    } else {
        FactClass::Ordinary
    })
}

pub fn map_semantic_operation_v2_with_context(
    semantic_operation: &[u8],
    assigned_sequence: u64,
    physical_ordinal: u64,
    catalog: &ReferenceCatalogV2,
    state: &AcceptedPrefixStateV2,
) -> Result<ContextualMappedRecordV2, ContextualMappingError> {
    // The encoded count is knowable without allocating the decoded target vector.
    let encoded_reference_count = fields(semantic_operation, b"RDOS-SOP2", 13)
        .and_then(|s| fields(s[8], b"RDOS-ENV2", 13))
        .and_then(|env| env[12].get(..4))
        .and_then(|v| <[u8; 4]>::try_from(v).ok())
        .map(u32::from_be_bytes);
    if encoded_reference_count.is_some_and(|count| count > 65_536) {
        return Err(ContextualMappingError::ResourceLimit);
    }
    let decoded = validate_semantic_operation_v2(semantic_operation)
        .map_err(ContextualMappingError::SemanticValidation)?;
    if decoded.references.len() > 65_536 {
        return Err(ContextualMappingError::ResourceLimit);
    }
    let pos = usize::try_from(state.accepted).map_err(|_| ContextualMappingError::Exhaustion)?;
    let expected = catalog
        .selected_operations
        .get(pos)
        .ok_or(ContextualMappingError::Exhaustion)?;
    if state.scope_id != catalog.scope_id || state.namespace != catalog.selected {
        return Err(ContextualMappingError::Discontinuity);
    }
    let offered = extract(semantic_operation).ok_or(ContextualMappingError::Extraction)?;
    if offered.bytes != expected.bytes
        || offered.namespace != expected.namespace
        || offered.segment != expected.segment
        || offered.ordinal != expected.ordinal
        || offered.producer != expected.producer
        || offered.producer_ordinal != expected.producer_ordinal
        || offered.request != expected.request
        || offered.event != expected.event
        || offered.information != expected.information
    {
        return Err(ContextualMappingError::Discontinuity);
    }
    for target in &offered.references {
        if *target == offered.event {
            return Err(ContextualMappingError::Reference(
                ReferenceError::SelfReference,
            ));
        }
        let Some(x) = catalog.identities.get(target) else {
            return Err(ContextualMappingError::Reference(ReferenceError::Missing));
        };
        debug_assert_eq!(x.id, *target);
        let _provenance = (x.total_position, x.producer, x.producer_ordinal);
        if x.role != Role::Event {
            return Err(ContextualMappingError::Reference(ReferenceError::WrongKind));
        }
        if x.fact != Some(FactClass::Ordinary) {
            return Err(ContextualMappingError::Reference(ReferenceError::WrongFact));
        }
        if x.namespace != offered.namespace {
            return Err(ContextualMappingError::Reference(
                ReferenceError::CrossStream,
            ));
        }
        if x.segment != offered.segment {
            return Err(ContextualMappingError::Reference(
                ReferenceError::CrossSegment,
            ));
        }
        if x.ordinal == offered.ordinal {
            return Err(ContextualMappingError::Reference(
                ReferenceError::SelfReference,
            ));
        }
        if x.ordinal > offered.ordinal {
            return Err(ContextualMappingError::Reference(ReferenceError::Future));
        }
    }
    let mapped = map_validated_core(
        semantic_operation,
        decoded.event_id,
        assigned_sequence,
        physical_ordinal,
        state.sequence,
        state.ordinal,
    )
    .map_err(ContextualMappingError::Mapping)?;
    let next = AcceptedPrefixStateV2 {
        accepted: state
            .accepted
            .checked_add(1)
            .ok_or(ContextualMappingError::ResourceLimit)?,
        sequence: assigned_sequence,
        ordinal: physical_ordinal,
        scope_id: state.scope_id,
        namespace: state.namespace,
    };
    Ok(ContextualMappedRecordV2 {
        frame: mapped.frame,
        record: mapped.record,
        next,
    })
}
