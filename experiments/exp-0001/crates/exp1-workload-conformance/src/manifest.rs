use super::{
    Error, artifact_digest, hex, manifest_digest, parse_uuid, stream_bindings, validate_stream,
    workload_digest,
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Eq, PartialEq)]
enum Json {
    String(String),
    Array(Vec<Json>),
    Object(BTreeMap<String, Json>),
    Bool,
    Null,
    Number,
}

struct Parser<'a> {
    b: &'a [u8],
    p: usize,
}
impl<'a> Parser<'a> {
    fn value(&mut self) -> Result<Json, Error> {
        match self.b.get(self.p) {
            Some(b'"') => self.string().map(Json::String),
            Some(b'{') => self.object(),
            Some(b'[') => self.array(),
            Some(b't') if self.take(b"true") => Ok(Json::Bool),
            Some(b'f') if self.take(b"false") => Ok(Json::Bool),
            Some(b'n') if self.take(b"null") => Ok(Json::Null),
            Some(b'-' | b'0'..=b'9') => {
                while matches!(
                    self.b.get(self.p),
                    Some(b'0'..=b'9' | b'-' | b'+' | b'.' | b'e' | b'E')
                ) {
                    self.p += 1;
                }
                Ok(Json::Number)
            }
            _ => Err(Error::JsonSyntax),
        }
    }
    fn take(&mut self, s: &[u8]) -> bool {
        if self.b.get(self.p..self.p + s.len()) == Some(s) {
            self.p += s.len();
            true
        } else {
            false
        }
    }
    fn string(&mut self) -> Result<String, Error> {
        self.p += 1;
        let mut s = String::new();
        loop {
            let c = *self.b.get(self.p).ok_or(Error::JsonSyntax)?;
            self.p += 1;
            match c {
                b'"' => return Ok(s),
                0..=0x1f => return Err(Error::JsonSyntax),
                b'\\' => {
                    let e = *self.b.get(self.p).ok_or(Error::JsonSyntax)?;
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
                            let h = self.b.get(self.p..self.p + 4).ok_or(Error::JsonSyntax)?;
                            self.p += 4;
                            let t = std::str::from_utf8(h).map_err(|_| Error::JsonSyntax)?;
                            let n = u16::from_str_radix(t, 16).map_err(|_| Error::JsonSyntax)?;
                            if (0xd800..=0xdfff).contains(&n) {
                                return Err(Error::JsonSyntax);
                            }
                            s.push(char::from_u32(n.into()).ok_or(Error::JsonSyntax)?)
                        }
                        _ => return Err(Error::JsonSyntax),
                    }
                }
                0x20..=0x7f => s.push(c as char),
                _ => {
                    self.p -= 1;
                    let rest =
                        std::str::from_utf8(&self.b[self.p..]).map_err(|_| Error::JsonSyntax)?;
                    let ch = rest.chars().next().ok_or(Error::JsonSyntax)?;
                    s.push(ch);
                    self.p += ch.len_utf8();
                }
            }
        }
    }
    fn object(&mut self) -> Result<Json, Error> {
        self.p += 1;
        let mut m = BTreeMap::new();
        if self.b.get(self.p) == Some(&b'}') {
            self.p += 1;
            return Ok(Json::Object(m));
        }
        loop {
            if self.b.get(self.p) != Some(&b'"') {
                return Err(Error::JsonSyntax);
            }
            let k = self.string()?;
            if self.b.get(self.p) != Some(&b':') {
                return Err(Error::JsonSyntax);
            }
            self.p += 1;
            let v = self.value()?;
            if m.insert(k, v).is_some() {
                return Err(Error::DuplicateMember);
            }
            match self.b.get(self.p) {
                Some(b',') => self.p += 1,
                Some(b'}') => {
                    self.p += 1;
                    break;
                }
                _ => return Err(Error::JsonSyntax),
            }
        }
        Ok(Json::Object(m))
    }
    fn array(&mut self) -> Result<Json, Error> {
        self.p += 1;
        let mut a = vec![];
        if self.b.get(self.p) == Some(&b']') {
            self.p += 1;
            return Ok(Json::Array(a));
        }
        loop {
            a.push(self.value()?);
            match self.b.get(self.p) {
                Some(b',') => self.p += 1,
                Some(b']') => {
                    self.p += 1;
                    break;
                }
                _ => return Err(Error::JsonSyntax),
            }
        }
        Ok(Json::Array(a))
    }
}
fn quote(s: &str, out: &mut String) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\x08' => out.push_str("\\b"),
            '\x0c' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"')
}
fn canonical(v: &Json, out: &mut String) {
    match v {
        Json::String(s) => quote(s, out),
        Json::Array(a) => {
            out.push('[');
            for (i, v) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',')
                }
                canonical(v, out)
            }
            out.push(']')
        }
        Json::Object(m) => {
            out.push('{');
            for (i, (k, v)) in m.iter().enumerate() {
                if i > 0 {
                    out.push(',')
                }
                quote(k, out);
                out.push(':');
                canonical(v, out)
            }
            out.push('}')
        }
        Json::Bool => out.push_str("true"),
        Json::Null => out.push_str("null"),
        Json::Number => out.push('0'),
    }
}
fn parse(b: &[u8]) -> Result<Json, Error> {
    std::str::from_utf8(b).map_err(|_| Error::JsonSyntax)?;
    let parsed = b.strip_prefix(&[0xef, 0xbb, 0xbf]).unwrap_or(b);
    let parsed = parsed
        .strip_suffix(b"\n")
        .or_else(|| parsed.strip_suffix(b"\r"))
        .unwrap_or(parsed);
    let mut p = Parser { b: parsed, p: 0 };
    let v = p.value()?;
    if p.p != parsed.len() {
        return Err(Error::JsonSyntax);
    }
    Ok(v)
}
fn obj(v: &Json) -> Result<&BTreeMap<String, Json>, Error> {
    if let Json::Object(x) = v {
        Ok(x)
    } else {
        Err(Error::Type)
    }
}
fn arr(v: &Json) -> Result<&[Json], Error> {
    if let Json::Array(x) = v {
        Ok(x)
    } else {
        Err(Error::Type)
    }
}
fn strv(v: &Json) -> Result<&str, Error> {
    if let Json::String(x) = v {
        Ok(x)
    } else {
        Err(Error::Type)
    }
}
fn closed<'a>(v: &'a Json, names: &[&str]) -> Result<&'a BTreeMap<String, Json>, Error> {
    let o = obj(v)?;
    for k in o.keys() {
        if !names.contains(&k.as_str()) {
            return Err(Error::UnknownField);
        }
    }
    for n in names {
        if !o.contains_key(*n) {
            return Err(Error::MissingField);
        }
    }
    Ok(o)
}
fn dec(s: &str, signed: bool) -> Result<(), Error> {
    if s == "0" {
        return Ok(());
    }
    if signed && s == "-9223372036854775808" {
        return Ok(());
    }
    if let Some(stripped) = s.strip_prefix('-') {
        if !signed || stripped.starts_with('0') || stripped.parse::<i64>().is_err() {
            return Err(Error::Range);
        }
    } else if s.starts_with('0') || s.parse::<u64>().is_err() {
        return Err(Error::Range);
    }
    Ok(())
}
fn digest(s: &str) -> Result<(), Error> {
    if s.len() != 64
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f'))
    {
        Err(Error::Type)
    } else {
        Ok(())
    }
}
fn uuid(s: &str) -> Result<(), Error> {
    parse_uuid(s).map(|_| ())
}
fn uri(s: &str) -> Result<(), Error> {
    if (s.starts_with("https://") || s.starts_with("file:/")) && !s.contains(['?', '#']) {
        Ok(())
    } else {
        Err(Error::Reference)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manifest {
    root: Json,
}
impl Manifest {
    pub fn parse(candidate: &[u8]) -> Result<Self, Error> {
        let root = parse(candidate)?;
        let mut c = String::new();
        canonical(&root, &mut c);
        if c.as_bytes() != candidate {
            return Err(Error::Noncanonical);
        }
        schema(&root)?;
        Ok(Self { root })
    }
    pub fn canonical_bytes(&self) -> Vec<u8> {
        let mut s = String::new();
        canonical(&self.root, &mut s);
        s.into_bytes()
    }
}

#[derive(Clone, Debug)]
pub struct ManifestReference<'a> {
    pub artifact_id: &'a str,
    pub byte_length: u64,
    pub sha256: &'a str,
    pub uri: &'a str,
}
#[derive(Clone, Debug)]
pub struct ManifestDigestDescriptor<'a> {
    pub algorithm: &'a str,
    pub domain: &'a str,
    pub profile: &'a str,
    pub value: &'a str,
    pub manifest_ref: ManifestReference<'a>,
}
#[derive(Clone, Debug)]
pub struct SupersessionTarget<'a> {
    pub manifest_id: &'a str,
    pub workload_id: &'a str,
    pub supersedes: &'a [&'a str],
}
#[derive(Clone, Debug)]
pub struct ArtifactMetadata<'a> {
    pub artifact_id: &'a str,
    pub byte_length: u64,
    pub sha256: &'a str,
    pub uri: &'a str,
    pub role: &'a str,
    pub media_type: &'a str,
    pub created_by_record_id: &'a str,
}
#[derive(Clone, Debug)]
pub struct ProvenanceEdge<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub relation: &'a str,
}
#[derive(Clone, Debug)]
pub struct ValidationContext<'a> {
    pub stream: &'a [u8],
    pub descriptor: &'a ManifestDigestDescriptor<'a>,
    pub manifest_artifact_sha256: &'a str,
    pub targets: &'a [SupersessionTarget<'a>],
    pub artifact_manifest_bytes: &'a [u8],
    pub stream_artifact: &'a ArtifactMetadata<'a>,
    pub manifest_artifact: &'a ArtifactMetadata<'a>,
    pub provenance: &'a [ProvenanceEdge<'a>],
}

pub fn validate_manifest(candidate: &[u8], ctx: &ValidationContext<'_>) -> Result<Manifest, Error> {
    let m = Manifest::parse(candidate)?;
    let top = obj(&m.root)?;
    let stream = ctx.stream;
    let (n, w, me) = validate_stream(stream)?;
    let counts = obj(&top["counts"])?;
    if strv(&counts["operation_count"])?.parse::<u64>().ok() != Some(n)
        || strv(&counts["warm_up_operation_count"])?
            .parse::<u64>()
            .ok()
            != Some(w)
        || strv(&counts["measured_operation_count"])?
            .parse::<u64>()
            .ok()
            != Some(me)
    {
        return Err(Error::CountMismatch);
    }
    let sd = obj(&top["stream_digest"])?;
    if strv(&sd["value"])? != hex(&workload_digest(stream)) {
        return Err(Error::Digest);
    }
    let sr = obj(&top["stream_ref"])?;
    if strv(&sr["byte_length"])?.parse::<usize>().ok() != Some(stream.len())
        || strv(&sr["sha256"])? != hex(&artifact_digest(stream))
    {
        return Err(Error::Reference);
    }
    let d = ctx.descriptor;
    if d.algorithm != "SHA-256/FIPS-180-4"
        || d.domain != "rusty-data-os/exp1/workload-manifest/v1"
        || d.profile != "EXP-0001-WORKLOAD-MANIFEST-DIGEST-v1"
        || d.value != hex(&manifest_digest(candidate))
    {
        return Err(Error::Digest);
    }
    if d.manifest_ref.artifact_id != strv(&top["manifest_id"])?
        || d.manifest_ref.byte_length as usize != candidate.len()
        || d.manifest_ref.sha256 != ctx.manifest_artifact_sha256
        || d.manifest_ref.sha256 != hex(&artifact_digest(candidate))
        || digest(d.manifest_ref.sha256).is_err()
        || uri(d.manifest_ref.uri).is_err()
    {
        return Err(Error::Reference);
    }
    validate_r7(top, candidate, ctx)?;
    validate_super(top, ctx.targets)?;
    validate_counts_and_stream(top, stream)?;
    Ok(m)
}

fn validate_r7(
    top: &BTreeMap<String, Json>,
    candidate: &[u8],
    ctx: &ValidationContext<'_>,
) -> Result<(), Error> {
    let sr = obj(&top["stream_ref"])?;
    let amr = obj(&sr["artifact_manifest_ref"])?;
    let stream = ctx.stream_artifact;
    if stream.artifact_id != strv(&sr["artifact_id"])?
        || stream.byte_length.to_string() != strv(&sr["byte_length"])?
        || stream.sha256 != strv(&sr["sha256"])?
        || stream.uri != strv(&sr["uri"])?
        || stream.role != strv(&sr["role"])?
        || stream.media_type != strv(&sr["media_type"])?
        || stream.created_by_record_id != strv(&sr["created_by_record_id"])?
        || ctx.artifact_manifest_bytes.len().to_string() != strv(&amr["byte_length"])?
        || hex(&artifact_digest(ctx.artifact_manifest_bytes)) != strv(&amr["sha256"])?
    {
        return Err(Error::Reference);
    }
    let ma = ctx.manifest_artifact;
    let d = &ctx.descriptor.manifest_ref;
    if ma.artifact_id != d.artifact_id
        || ma.byte_length as usize != candidate.len()
        || ma.sha256 != hex(&artifact_digest(candidate))
        || ma.uri != d.uri
        || ma.role != "configuration"
        || ma.media_type != "application/vnd.rusty-data-os.exp1-workload-manifest"
    {
        return Err(Error::Reference);
    }
    let created = stream.created_by_record_id;
    if !ctx
        .provenance
        .iter()
        .any(|e| e.from == stream.artifact_id && e.to == created && e.relation == "created_by")
    {
        return Err(Error::Reference);
    }
    let mut current = created;
    let mut seen = BTreeSet::new();
    while current != ma.artifact_id {
        if !seen.insert(current) {
            return Err(Error::Reference);
        }
        let next = ctx
            .provenance
            .iter()
            .find(|e| e.from == current && e.relation == "derived_from")
            .ok_or(Error::Reference)?;
        current = next.to;
    }
    Ok(())
}

fn schema(v: &Json) -> Result<(), Error> {
    let t = closed(
        v,
        &[
            "authority_revisions",
            "counts",
            "created_at_utc_ns",
            "generator_inputs",
            "manifest_id",
            "profiles",
            "record_kind",
            "schema_version",
            "stream_digest",
            "stream_ref",
            "supersession",
            "workload_id",
        ],
    )?;
    if strv(&t["schema_version"])? != "EXP-0001-WORKLOAD-MANIFEST-JCS-v1" {
        return Err(Error::Unsupported);
    }
    if strv(&t["record_kind"])? != "workload_manifest" {
        return Err(Error::Unsupported);
    }
    uuid(strv(&t["manifest_id"])?)?;
    uuid(strv(&t["workload_id"])?)?;
    dec(strv(&t["created_at_utc_ns"])?, true)?;
    let p = closed(
        &t["profiles"],
        &[
            "digest",
            "envelope",
            "envelope_generator",
            "identity_generator",
            "logical_time_generator",
            "manifest",
            "payload_content",
            "payload_generator",
            "payload_size",
            "reference_generator",
            "semantic_operation",
            "size_class_order",
            "temporal",
            "workload_contract",
            "workload_stream",
        ],
    )?;
    for (k, x) in [
        ("digest", "SHA-256/FIPS-180-4"),
        ("manifest", "EXP-0001-WORKLOAD-MANIFEST-JCS-v1"),
        ("identity_generator", "EXP-0001-UUID4-SHA256-v1"),
        ("envelope_generator", "EXP-0001-ENVELOPE-INPUT-v1"),
        ("reference_generator", "EXP-0001-PRIOR-EVENTS-v1"),
        ("logical_time_generator", "EXP-0001-LOGICAL-TIME-v1"),
        ("semantic_operation", "EXP-0001-SEMANTIC-OP-v1"),
        ("workload_stream", "EXP-0001-WORKLOAD-STREAM-v1"),
        ("workload_contract", "EXP-0000-WORKLOADS-v1"),
        ("size_class_order", "EXP-0000-SIZE-CLASS-ORDER-v1"),
    ] {
        if strv(&p[k])? != x {
            return Err(Error::ProfileMismatch);
        }
    }
    let payload_generator = strv(&p["payload_generator"])?;
    let payload_content = strv(&p["payload_content"])?;
    let expected_content = match payload_generator {
        "EXP-0001-SHA256-CTR-v1" => "deterministic-high-variation",
        "EXP-0001-SHA256-MOTIF-v1" => "repeated-low-variation",
        "EXP-0001-ZERO-v1" => "all-zero",
        _ => return Err(Error::ProfileMismatch),
    };
    if payload_content != expected_content
        || !matches!(
            strv(&p["payload_size"])?,
            "fixed-P0"
                | "fixed-P1"
                | "fixed-P2"
                | "fixed-P3"
                | "fixed-P4"
                | "fixed-P5"
                | "mixed-equal-P1-P4"
                | "mixed-weighted-P1-P4-v1"
        )
        || !matches!(
            strv(&p["envelope"])?,
            "envelope-minimal"
                | "envelope-provenance"
                | "envelope-causal-reference"
                | "envelope-correction-retraction-reference"
        )
        || !matches!(
            strv(&p["temporal"])?,
            "time-monotonic-effective"
                | "time-equal-burst-v1"
                | "time-late-arriving-v1"
                | "time-out-of-effective-order-v1"
        )
    {
        return Err(Error::ProfileMismatch);
    }
    let g = closed(
        &t["generator_inputs"],
        &[
            "actor_provenance",
            "base_ns",
            "controlled_schedule",
            "correction_fact_type",
            "envelope_semantic_version",
            "generator_version",
            "ordinary_fact_type",
            "producer_count",
            "producer_id",
            "reference_cardinality",
            "schema_id",
            "schema_version",
            "seed",
            "source_provenance",
            "stream_namespace",
            "unit_ns",
            "workload_contract_version",
        ],
    )?;
    for k in ["producer_id", "schema_id", "stream_namespace"] {
        uuid(strv(&g[k])?)?
    }
    for k in [
        "generator_version",
        "producer_count",
        "reference_cardinality",
        "seed",
        "workload_contract_version",
    ] {
        dec(strv(&g[k])?, false)?
    }
    for k in ["base_ns", "unit_ns"] {
        dec(strv(&g[k])?, true)?
    }
    if strv(&g["generator_version"])? != "1"
        || strv(&g["producer_count"])? != "1"
        || strv(&g["workload_contract_version"])? != "1"
        || strv(&g["unit_ns"])?.parse::<i64>().map_or(true, |x| x <= 0)
    {
        return Err(Error::ProfileMismatch);
    }
    let actor = string_state(&g["actor_provenance"])?;
    let source = string_state(&g["source_provenance"])?;
    let correction = string_state(&g["correction_fact_type"])?;
    let schedule = uuid_state(&g["controlled_schedule"])?;
    for k in [
        "schema_version",
        "envelope_semantic_version",
        "ordinary_fact_type",
    ] {
        if strv(&g[k])?.is_empty() {
            return Err(Error::Type);
        }
    }
    let refs = strv(&g["reference_cardinality"])?
        .parse::<u64>()
        .map_err(|_| Error::Range)?;
    let applicable = match strv(&p["envelope"])? {
        "envelope-minimal" => {
            source.is_none() && actor.is_none() && correction.is_none() && refs == 0
        }
        "envelope-provenance" => {
            source.is_some() && actor.is_some() && correction.is_none() && refs == 0
        }
        "envelope-causal-reference" => refs > 0 && correction.is_none(),
        "envelope-correction-retraction-reference" => refs > 0 && correction.is_some(),
        _ => false,
    };
    if !applicable {
        return Err(Error::ProfileMismatch);
    }
    let c = closed(
        &t["counts"],
        &[
            "by_envelope_profile",
            "by_segment",
            "by_size_class",
            "by_temporal_profile",
            "measured_operation_count",
            "operation_count",
            "warm_up_operation_count",
        ],
    )?;
    for k in [
        "measured_operation_count",
        "operation_count",
        "warm_up_operation_count",
    ] {
        dec(strv(&c[k])?, false)?
    }
    for k in [
        "by_envelope_profile",
        "by_segment",
        "by_size_class",
        "by_temporal_profile",
    ] {
        for e in arr(&c[k])? {
            let o = obj(e)?;
            if o.len() != 2
                || !o.contains_key("count")
                || (!o.contains_key("profile") && !o.contains_key("segment"))
            {
                return Err(Error::UnknownField);
            }
            dec(strv(&o["count"])?, false)?
        }
    }
    let sd = closed(&t["stream_digest"], &["algorithm", "domain", "value"])?;
    if strv(&sd["algorithm"])? != "SHA-256/FIPS-180-4"
        || strv(&sd["domain"])? != "rusty-data-os/exp1/workload-stream/v1"
    {
        return Err(Error::ProfileMismatch);
    }
    digest(strv(&sd["value"])?)?;
    reference(&t["stream_ref"])?;
    let s = closed(&t["supersession"], &["reason", "supersedes_manifest_ids"])?;
    state(&s["reason"])?;
    for x in arr(&s["supersedes_manifest_ids"])? {
        uuid(strv(x)?)?
    }
    let required = [
        "EXP-0000-WORKLOADS",
        "EXP-0001-R12",
        "EXP-0001-R14",
        "EXP-0001-R16",
        "EXP-0001-R2",
        "EXP-0001-R7",
    ];
    let authorities = arr(&t["authority_revisions"])?;
    if authorities.len() != required.len() {
        return Err(Error::MissingField);
    }
    let mut auth = BTreeSet::new();
    for (x, required_name) in authorities.iter().zip(required) {
        let a = closed(x, &["authority", "revision"])?;
        let name = strv(&a["authority"])?;
        if name != required_name || !auth.insert(name) {
            return Err(Error::Ordering);
        }
        let r = closed(&a["revision"], &["kind", "value"])?;
        let kind = strv(&r["kind"])?;
        let val = strv(&r["value"])?;
        if kind == "git_sha" {
            if val.len() != 40 || digest(&format!("{val}{}", "0".repeat(24))).is_err() {
                return Err(Error::Type);
            }
        } else if kind != "reviewed_authority_id"
            || val.is_empty()
            || (val.len() == 40
                && val
                    .bytes()
                    .all(|b| b.is_ascii_digit() || matches!(b, b'a'..=b'f')))
        {
            return Err(Error::Type);
        }
    }
    let _ = schedule;
    Ok(())
}
fn state(v: &Json) -> Result<(), Error> {
    let o = obj(v)?;
    match o.get("state").and_then(|x| strv(x).ok()) {
        Some("not_applicable") if o.len() == 1 => Ok(()),
        Some("present") if o.len() == 2 && o.contains_key("value") => Ok(()),
        _ => Err(Error::Type),
    }
}
fn string_state(v: &Json) -> Result<Option<&str>, Error> {
    state(v)?;
    let o = obj(v)?;
    if strv(&o["state"])? == "not_applicable" {
        Ok(None)
    } else {
        let x = strv(&o["value"])?;
        if x.is_empty() {
            Err(Error::Type)
        } else {
            Ok(Some(x))
        }
    }
}
fn uuid_state(v: &Json) -> Result<Option<[u8; 16]>, Error> {
    state(v)?;
    let o = obj(v)?;
    if strv(&o["state"])? == "not_applicable" {
        Ok(None)
    } else {
        Ok(Some(parse_uuid(strv(&o["value"])?)?))
    }
}
fn reference(v: &Json) -> Result<(), Error> {
    let o = closed(
        v,
        &[
            "artifact_id",
            "artifact_manifest_ref",
            "byte_length",
            "created_by_record_id",
            "media_type",
            "role",
            "sha256",
            "uri",
        ],
    )?;
    uuid(strv(&o["artifact_id"])?)?;
    uuid(strv(&o["created_by_record_id"])?)?;
    dec(strv(&o["byte_length"])?, false)?;
    digest(strv(&o["sha256"])?)?;
    uri(strv(&o["uri"])?)?;
    if strv(&o["role"])? != "configuration"
        || strv(&o["media_type"])? != "application/vnd.rusty-data-os.exp1-workload-stream"
    {
        return Err(Error::Reference);
    }
    let r = closed(
        &o["artifact_manifest_ref"],
        &["artifact_id", "byte_length", "sha256", "uri"],
    )?;
    uuid(strv(&r["artifact_id"])?)?;
    dec(strv(&r["byte_length"])?, false)?;
    digest(strv(&r["sha256"])?)?;
    uri(strv(&r["uri"])?)
}

fn profile_counts(v: &Json, key: &str) -> Result<BTreeMap<String, u64>, Error> {
    let mut out = BTreeMap::new();
    let mut previous: Option<&str> = None;
    for entry in arr(v)? {
        let o = closed(entry, &["count", key])?;
        let name = strv(&o[key])?;
        if name.is_empty() || previous.is_some_and(|p| name <= p) {
            return Err(Error::Ordering);
        }
        let count = strv(&o["count"])?
            .parse::<u64>()
            .map_err(|_| Error::Range)?;
        if count == 0 || out.insert(name.to_owned(), count).is_some() {
            return Err(Error::DuplicateOrConflict);
        }
        previous = Some(name);
    }
    Ok(out)
}

fn validate_counts_and_stream(top: &BTreeMap<String, Json>, stream: &[u8]) -> Result<(), Error> {
    let (total, warm, measured) = validate_stream(stream)?;
    let c = obj(&top["counts"])?;
    let segments = arr(&c["by_segment"])?;
    if segments.len() != 2 {
        return Err(Error::CountMismatch);
    }
    for (entry, name, count) in [
        (segments[0].clone(), "warm_up", warm),
        (segments[1].clone(), "measured", measured),
    ] {
        let o = closed(&entry, &["count", "segment"])?;
        if strv(&o["segment"])? != name || strv(&o["count"])?.parse::<u64>().ok() != Some(count) {
            return Err(Error::CountMismatch);
        }
    }
    let envelope = profile_counts(&c["by_envelope_profile"], "profile")?;
    let temporal = profile_counts(&c["by_temporal_profile"], "profile")?;
    let sizes = profile_counts(&c["by_size_class"], "profile")?;
    let sum = |m: &BTreeMap<String, u64>| {
        m.values()
            .try_fold(0u64, |a, x| a.checked_add(*x))
            .ok_or(Error::CountMismatch)
    };
    if (total == 0 && (!envelope.is_empty() || !temporal.is_empty() || !sizes.is_empty()))
        || (total > 0
            && (sum(&envelope)? != total || sum(&temporal)? != total || sum(&sizes)? != total))
    {
        return Err(Error::CountMismatch);
    }
    let actual = stream_bindings(stream)?;
    let actual_env: BTreeMap<String, u64> = actual
        .envelope
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    let actual_time: BTreeMap<String, u64> = actual
        .temporal
        .into_iter()
        .map(|(k, v)| (k.to_owned(), v))
        .collect();
    if envelope != actual_env || temporal != actual_time || sizes != actual.size {
        return Err(Error::CountMismatch);
    }
    let profiles = obj(&top["profiles"])?;
    if total > 0
        && (envelope.len() != 1
            || !envelope.contains_key(strv(&profiles["envelope"])?)
            || temporal.len() != 1
            || !temporal.contains_key(strv(&profiles["temporal"])?))
    {
        return Err(Error::ProfileMismatch);
    }
    let expected_sizes: Vec<String> = match strv(&profiles["payload_size"])? {
        x if x.starts_with("fixed-") => vec![x[6..].to_owned()],
        _ => sizes.keys().cloned().collect(),
    };
    if expected_sizes != sizes.keys().cloned().collect::<Vec<_>>() {
        return Err(Error::ProfileMismatch);
    }
    let inputs = obj(&top["generator_inputs"])?;
    let declared_schedule = uuid_state(&inputs["controlled_schedule"])?;
    if total > 0 && actual.schedule != Some(declared_schedule) {
        return Err(Error::ProfileMismatch);
    }
    Ok(())
}
fn validate_super(
    top: &BTreeMap<String, Json>,
    targets: &[SupersessionTarget<'_>],
) -> Result<(), Error> {
    let id = strv(&top["manifest_id"])?;
    let workload = strv(&top["workload_id"])?;
    let s = obj(&top["supersession"])?;
    let ids = arr(&s["supersedes_manifest_ids"])?;
    let reason = obj(&s["reason"])?;
    if ids.is_empty() {
        if strv(&reason["state"])? != "not_applicable" {
            return Err(Error::ImmutableState);
        }
        return Ok(());
    }
    if strv(&reason["state"])? != "present" {
        return Err(Error::ImmutableState);
    }
    let mut prev = "";
    for x in ids {
        let x = strv(x)?;
        if x <= prev || x == id {
            return Err(Error::DuplicateOrConflict);
        }
        let t = targets
            .iter()
            .find(|t| t.manifest_id == x)
            .ok_or(Error::Reference)?;
        if t.workload_id != workload || t.supersedes.contains(&id) {
            return Err(Error::SupersessionCycle);
        }
        prev = x
    }
    Ok(())
}
