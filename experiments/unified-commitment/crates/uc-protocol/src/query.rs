use crate::*;
use std::cmp::Ordering;

fn kind(schema: &DomainSchema, tag: FieldRef) -> Result<ValueKind, ErrorCode> {
    schema
        .fields
        .iter()
        .find(|f| f.tag == tag)
        .map(|f| f.value_kind)
        .ok_or(ErrorCode::UnknownField)
}
fn matches_kind(kind: ValueKind, value: &ScanValue) -> bool {
    matches!(
        (kind, value),
        (ValueKind::U32, ScanValue::U32(_))
            | (ValueKind::I64, ScanValue::I64(_))
            | (ValueKind::Bool, ScanValue::Bool(_))
            | (ValueKind::Str, ScanValue::Str(_))
    )
}
pub fn validate_predicate(schema: &DomainSchema, predicate: &Predicate) -> Result<(), ErrorCode> {
    let k = kind(schema, predicate.field)?;
    if !matches_kind(k, &predicate.value)
        || (predicate.op.is_ordering() && !matches!(k, ValueKind::U32 | ValueKind::I64))
    {
        return Err(ErrorCode::Malformed);
    }
    Ok(())
}
pub fn validate_query(
    schema: &DomainSchema,
    selection: &Selection,
    filter: &[Predicate],
) -> Result<(), ErrorCode> {
    if let Selection::Fields(tags) = selection {
        for &tag in tags {
            kind(schema, tag)?;
        }
    }
    for pred in filter {
        validate_predicate(schema, pred)?;
    }
    Ok(())
}
pub fn predicate_matches(fields: &[(FieldRef, ScanValue)], predicate: &Predicate) -> bool {
    let Some((_, actual)) = fields.iter().find(|(tag, _)| *tag == predicate.field) else {
        return false;
    };
    match predicate.op {
        CompareOp::Eq => actual == &predicate.value,
        CompareOp::Ne => actual != &predicate.value,
        op => {
            let order = match (actual, &predicate.value) {
                (ScanValue::U32(a), ScanValue::U32(b)) => a.cmp(b),
                (ScanValue::I64(a), ScanValue::I64(b)) => a.cmp(b),
                _ => return false,
            };
            matches!(
                (op, order),
                (CompareOp::Lt, Ordering::Less)
                    | (CompareOp::Le, Ordering::Less | Ordering::Equal)
                    | (CompareOp::Gt, Ordering::Greater)
                    | (CompareOp::Ge, Ordering::Greater | Ordering::Equal)
            )
        }
    }
}
fn project(fields: Fields, selection: &Selection) -> Fields {
    match selection {
        Selection::All => fields,
        Selection::Fields(tags) => fields
            .into_iter()
            .filter(|(tag, _)| tags.contains(tag))
            .collect(),
    }
}
pub(crate) fn evaluate_query(
    rows: Vec<PageRow>,
    selection: &Selection,
    filter: &[Predicate],
    limit: Option<usize>,
) -> Vec<PageRow> {
    rows.into_iter()
        .filter(|(_, fields)| filter.iter().all(|pred| predicate_matches(fields, pred)))
        .take(limit.unwrap_or(usize::MAX))
        .map(|(id, fields)| (id, project(fields, selection)))
        .collect()
}
pub fn default_relation_descriptors(
    schema: &DomainSchema,
    labels: Vec<String>,
) -> Vec<RelationDescriptor> {
    if !schema.relations.neighbors {
        return Vec::new();
    }
    let mut relations = vec![RelationDescriptor {
        name: "neighbors".into(),
        kind: JoinRelation::Neighbors(None),
        target_table: None,
    }];
    relations.extend(labels.into_iter().map(|label| RelationDescriptor {
        name: label.clone(),
        kind: JoinRelation::Neighbors(Some(label)),
        target_table: None,
    }));
    relations
}
/// Preserve each wire error distinction, including unresolved named right tables.
pub fn validate_join(
    schema: &DomainSchema,
    relations: &[RelationDescriptor],
    right_schema: Option<&DomainSchema>,
    spec: &JoinSpec,
) -> Result<(), ErrorCode> {
    validate_query(schema, &spec.left, &spec.left_filter)?;
    let relation = relations
        .iter()
        .find(|r| r.kind == spec.relation)
        .ok_or(ErrorCode::Malformed)?;
    match (&spec.right_table, &relation.target_table) {
        (None, None) => validate_query(schema, &spec.right, &spec.right_filter),
        (None, Some(_)) => Err(ErrorCode::Unsupported),
        (Some(named), Some(target)) if named == target => validate_query(
            right_schema.ok_or(ErrorCode::Malformed)?,
            &spec.right,
            &spec.right_filter,
        ),
        (Some(_), _) => Err(ErrorCode::Malformed),
    }
}
pub(crate) fn evaluate_join<L: Store + ?Sized, R: Store + ?Sized>(
    left: &L,
    right: &R,
    spec: &JoinSpec,
) -> Vec<JoinedRow> {
    let mut rows = Vec::new();
    let limit = spec.limit.unwrap_or(usize::MAX);
    if limit == 0 {
        return rows;
    }
    for (left_id, fields) in left.scan_all() {
        if !spec
            .left_filter
            .iter()
            .all(|p| predicate_matches(&fields, p))
        {
            continue;
        }
        let ids = match &spec.relation {
            JoinRelation::Neighbors(None) => left.neighbors(left_id).unwrap_or_default(),
            JoinRelation::Neighbors(Some(label)) => left
                .neighbors_by_relation(left_id, label)
                .unwrap_or_default(),
            JoinRelation::Children => left.children(left_id).unwrap_or_default(),
            JoinRelation::Parent => match left.parent(left_id) {
                Ok(ParentLookup::Parent(id)) => vec![id],
                _ => Vec::new(),
            },
        };
        for right_id in ids {
            if let Some(right_fields) = right.get(right_id)
                && spec
                    .right_filter
                    .iter()
                    .all(|p| predicate_matches(&right_fields, p))
            {
                rows.push(JoinedRow {
                    left_id,
                    left: project(fields.clone(), &spec.left),
                    right_id,
                    right: project(right_fields, &spec.right),
                });
                if rows.len() == limit {
                    return rows;
                }
            }
        }
    }
    rows
}
pub(crate) fn validate_page(
    schema: &DomainSchema,
    tag: FieldRef,
    after: Option<&(ScanValue, RecordId)>,
    limit: u64,
) -> Result<(), ErrorCode> {
    let k = kind(schema, tag)?;
    if limit == 0
        || !matches!(k, ValueKind::U32 | ValueKind::I64)
        || after.is_some_and(|(v, _)| !matches_kind(k, v))
    {
        return Err(ErrorCode::Malformed);
    }
    Ok(())
}
pub fn page_key_value(value: &ScanValue) -> Option<i128> {
    match value {
        ScanValue::U32(v) => Some(i128::from(*v)),
        ScanValue::I64(v) => Some(i128::from(*v)),
        _ => None,
    }
}
pub fn page_key(fields: &[(FieldRef, ScanValue)], tag: FieldRef, id: RecordId) -> (i128, RecordId) {
    (
        fields
            .iter()
            .find(|(f, _)| *f == tag)
            .and_then(|(_, v)| page_key_value(v))
            .unwrap_or(i128::MIN),
        id,
    )
}
pub fn page_ids(
    mut keys: Vec<(RecordId, i128)>,
    after: Option<(ScanValue, RecordId)>,
    limit: usize,
) -> Vec<RecordId> {
    let cursor = after.and_then(|(v, id)| page_key_value(&v).map(|v| (v, id)));
    keys.retain(|(id, key)| cursor.is_none_or(|c| (*key, *id) > c));
    keys.sort_by_key(|(id, key)| (*key, *id));
    keys.into_iter().take(limit).map(|(id, _)| id).collect()
}
pub fn page_by_scan<S: Store + ?Sized>(
    store: &S,
    tag: FieldRef,
    after: Option<(ScanValue, RecordId)>,
    limit: usize,
) -> Vec<PageRow> {
    page_ids(store.page_keys(tag), after, limit)
        .into_iter()
        .filter_map(|id| store.get(id).map(|f| (id, f)))
        .collect()
}
pub fn page_rows(
    mut rows: Vec<PageRow>,
    tag: FieldRef,
    after: Option<(ScanValue, RecordId)>,
    limit: usize,
) -> Vec<PageRow> {
    let cursor = after.and_then(|(v, id)| page_key_value(&v).map(|v| (v, id)));
    rows.retain(|(id, f)| cursor.is_none_or(|c| page_key(f, tag, *id) > c));
    rows.sort_by_key(|(id, f)| page_key(f, tag, *id));
    rows.truncate(limit);
    rows
}
pub(crate) fn validate_aggregate(
    schema: &DomainSchema,
    group_by: &[FieldRef],
    filter: &[Predicate],
    specs: &[AggregateSpec],
) -> Result<(), ErrorCode> {
    for &tag in group_by {
        if kind(schema, tag)? == ValueKind::StrList {
            return Err(ErrorCode::Malformed);
        }
    }
    validate_query(schema, &Selection::All, filter)?;
    for spec in specs {
        match (spec.func, spec.field) {
            (AggregateFn::Count, None) => {}
            (AggregateFn::Count, Some(_)) | (_, None) => return Err(ErrorCode::Malformed),
            (_, Some(tag)) => {
                if !matches!(kind(schema, tag)?, ValueKind::U32 | ValueKind::I64) {
                    return Err(ErrorCode::Malformed);
                }
            }
        }
    }
    Ok(())
}
pub(crate) fn evaluate_aggregate(
    rows: Vec<PageRow>,
    group_by: &[FieldRef],
    filter: &[Predicate],
    specs: &[AggregateSpec],
    limit: Option<usize>,
    schema: &DomainSchema,
) -> Result<Vec<AggregateGroup>, ErrorCode> {
    let mut buckets: Vec<(Fields, Vec<Fields>)> = if group_by.is_empty() {
        vec![(Vec::new(), Vec::new())]
    } else {
        Vec::new()
    };
    for (_, fields) in rows
        .into_iter()
        .filter(|(_, f)| filter.iter().all(|p| predicate_matches(f, p)))
    {
        let key: Fields = group_by
            .iter()
            .filter_map(|tag| fields.iter().find(|(f, _)| f == tag).cloned())
            .collect();
        if let Some((_, members)) = buckets.iter_mut().find(|(k, _)| k == &key) {
            members.push(fields);
        } else {
            buckets.push((key, vec![fields]));
        }
    }
    buckets
        .into_iter()
        .take(limit.unwrap_or(usize::MAX))
        .map(|(key, rows)| {
            Ok(AggregateGroup {
                key,
                values: specs
                    .iter()
                    .map(|spec| reduce(spec, &rows, schema))
                    .collect::<Result<_, _>>()?,
            })
        })
        .collect()
}
fn reduce(
    spec: &AggregateSpec,
    rows: &[Fields],
    schema: &DomainSchema,
) -> Result<ScanValue, ErrorCode> {
    if spec.func == AggregateFn::Count {
        return Ok(ScanValue::I64(rows.len() as i64));
    }
    let tag = spec.field.unwrap_or_default();
    let values: Vec<&ScanValue> = rows
        .iter()
        .filter_map(|f| f.iter().find(|(t, _)| *t == tag).map(|(_, v)| v))
        .collect();
    Ok(match spec.func {
        AggregateFn::Count => ScanValue::I64(rows.len() as i64),
        AggregateFn::Sum | AggregateFn::Avg => {
            // Wide accumulation preserves Avg fractions and makes Sum overflow a response.
            let sum = values
                .iter()
                .map(|v| page_key_value(v).unwrap_or(0))
                .sum::<i128>();
            if spec.func == AggregateFn::Sum {
                ScanValue::I64(i64::try_from(sum).map_err(|_| ErrorCode::Malformed)?)
            } else {
                ScanValue::F64(if values.is_empty() {
                    0.0
                } else {
                    sum as f64 / values.len() as f64
                })
            }
        }
        AggregateFn::Min | AggregateFn::Max => {
            let best = if spec.func == AggregateFn::Min {
                values.into_iter().min_by_key(|v| page_key_value(v))
            } else {
                values.into_iter().max_by_key(|v| page_key_value(v))
            };
            best.cloned().unwrap_or_else(|| {
                if kind(schema, tag) == Ok(ValueKind::U32) {
                    ScanValue::U32(0)
                } else {
                    ScanValue::I64(0)
                }
            })
        }
    })
}
