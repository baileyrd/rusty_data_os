//! EXP-0005 Step 4b-ii: independently committed, same-table domain facades.
//! Each Store owns an already opened engine, preserving its explicit D1/D2 setting.
//! No wire request identity/retry or cross-domain commitment guarantee is added.
mod entity;
mod listener;
mod memory;
mod relation;

pub use entity::EntityStore;
pub use listener::{LoopbackListener, serve};
pub use memory::MemoryStore;
pub use relation::RelationStore;

use uc_core::{LogError, Outcome, Rejection};
use uc_protocol::*;

fn schema(
    fields: &[(&str, ValueKind)],
    filters: &[FieldRef],
    update: FieldRef,
    neighbors: bool,
) -> DomainSchema {
    DomainSchema {
        fields: fields
            .iter()
            .enumerate()
            .map(|(i, (name, kind))| FieldDescriptor {
                tag: i as FieldRef,
                name: (*name).into(),
                value_kind: *kind,
                capabilities: FieldCapabilities {
                    filter_eq: filters.contains(&(i as FieldRef)),
                    scan: i == usize::from(update),
                    update: i == usize::from(update),
                },
            })
            .collect(),
        relations: RelationCapabilities {
            parent_children: false,
            neighbors,
        },
    }
}

/// Exact complete fields, normalized by tag. Wire order need not be tag order.
fn decode_fields(fields: Fields, schema: &DomainSchema) -> Result<Vec<ScanValue>, ErrorCode> {
    if fields.len() != schema.fields.len() {
        return Err(ErrorCode::Malformed);
    }
    let mut values = vec![None; schema.fields.len()];
    for (tag, value) in fields {
        let index = usize::from(tag);
        let Some(field) = schema.fields.get(index) else {
            return Err(ErrorCode::Malformed);
        };
        if values[index].is_some()
            || !matches!(
                (field.value_kind, &value),
                (ValueKind::Str, ScanValue::Str(_))
                    | (ValueKind::StrList, ScanValue::StrList(_))
                    | (ValueKind::I64, ScanValue::I64(_))
                    | (ValueKind::Bool, ScanValue::Bool(_))
            )
        {
            return Err(ErrorCode::Malformed);
        }
        values[index] = Some(value);
    }
    values
        .into_iter()
        .map(|v| v.ok_or(ErrorCode::Malformed))
        .collect()
}

fn integer_update(
    field: FieldRef,
    value: &ScanValue,
    allowed: FieldRef,
    nonnegative: bool,
) -> Result<i64, ErrorCode> {
    if field != allowed {
        return Err(ErrorCode::Unsupported);
    }
    match value {
        ScanValue::I64(v) if !nonnegative || *v >= 0 => Ok(*v),
        _ => Err(ErrorCode::Malformed),
    }
}

fn check_reads(
    reads: &[(RecordId, FieldRef, ScanValue)],
    get: impl Fn(RecordId) -> Option<Fields>,
) -> Result<(), (usize, ErrorCode)> {
    for (id, field, expected) in reads {
        if get(*id)
            .and_then(|f| f.into_iter().find(|(tag, _)| tag == field).map(|(_, v)| v))
            .as_ref()
            != Some(expected)
        {
            return Err((0, ErrorCode::Conflict));
        }
    }
    Ok(())
}

fn committed(outcome: Result<Outcome, LogError>) -> Result<(), ErrorCode> {
    match outcome {
        Ok(Outcome::Committed { .. }) => Ok(()),
        Ok(Outcome::Rejected {
            reason: Rejection::Validation(message),
        }) => Err(match message.as_str() {
            "NotFound" | "RecordNotFound" => ErrorCode::RecordNotFound,
            "StaleIncarnation" => ErrorCode::Conflict,
            "Duplicate" => ErrorCode::Duplicate,
            "GuardFailed" => ErrorCode::GuardFailed,
            _ => ErrorCode::Malformed,
        }),
        // Storage also covers indeterminate outcomes; never represent them as a
        // precondition refusal or retry a request with a fresh identity.
        _ => Err(ErrorCode::Storage),
    }
}

fn text(value: &ScanValue) -> String {
    match value {
        ScanValue::Str(s) => s.clone(),
        _ => unreachable!("decoded string"),
    }
}
fn int(value: &ScanValue) -> i64 {
    match value {
        ScanValue::I64(v) => *v,
        _ => unreachable!("decoded integer"),
    }
}
fn strings(value: &ScanValue) -> Vec<String> {
    match value {
        ScanValue::StrList(v) => v.clone(),
        _ => unreachable!("decoded string list"),
    }
}
