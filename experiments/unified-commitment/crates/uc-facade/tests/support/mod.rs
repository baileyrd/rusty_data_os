#![allow(dead_code)]
use std::{
    fs,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};
use uc_core::Durability;
use uc_facade::{EntityStore, MemoryStore, RelationStore};
use uc_protocol::*;

pub struct Temp(pub PathBuf);
impl Temp {
    pub fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "uc-facade-{}-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    pub fn stores(&self, reopen: bool) -> Vec<Arc<dyn Store>> {
        let memory = if reopen {
            uc_memory::MemoryEngine::open(&self.0.join("memory"), Durability::D1)
        } else {
            uc_memory::MemoryEngine::create_with_label(
                &self.0.join("memory"),
                Durability::D1,
                "facade correctness D1".into(),
            )
        }
        .unwrap()
        .0;
        let entity = if reopen {
            uc_entity::EntityEngine::open(&self.0.join("entity"), Durability::D1)
        } else {
            uc_entity::EntityEngine::create(&self.0.join("entity"), Durability::D1)
        }
        .unwrap()
        .0;
        let relation = if reopen {
            uc_relation::RelationEngine::open(&self.0.join("relation"), Durability::D1)
        } else {
            uc_relation::RelationEngine::create(&self.0.join("relation"), Durability::D1)
        }
        .unwrap()
        .0;
        let entity: Arc<dyn Store> = Arc::new(EntityStore::new(entity));
        vec![
            Arc::new(MemoryStore::new(memory, entity.clone())),
            entity,
            Arc::new(RelationStore::new(relation)),
        ]
    }
}
impl Drop for Temp {
    fn drop(&mut self) {
        let path = self.0.canonicalize().unwrap();
        assert_eq!(
            path.parent().unwrap(),
            std::env::temp_dir().canonicalize().unwrap()
        );
        assert!(
            path.file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with("uc-facade-")
        );
        fs::remove_dir_all(path).unwrap();
    }
}
pub fn registry(stores: &[Arc<dyn Store>]) -> Registry {
    Registry::new(
        stores
            .iter()
            .map(|s| (s.table_name().into(), s.clone()))
            .collect(),
        0,
    )
    .unwrap()
}
pub const UPDATE: [FieldRef; 3] = [10, 2, 4];
pub fn id(n: u8) -> RecordId {
    RecordId([
        n, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd, 0xef, 0x10, 0x32, 0x54, 0x76, 0x98, 0xba, 0xdc, 0xfe,
    ])
}
pub fn fields(domain: usize, count: i64) -> Fields {
    use ScanValue::*;
    let values = match domain {
        0 => vec![
            Str("memory α".into()),
            Str("note".into()),
            StrList(vec!["one".into(), "二".into()]),
            Str("source".into()),
            Str("{\"x\":1}".into()),
            I64(-100),
            I64(200),
            Str("fact".into()),
            Str("active".into()),
            Bool(true),
            I64(count),
            I64(0),
            Str("node-m".into()),
        ],
        1 => vec![
            Str("Ada".into()),
            Str("person".into()),
            I64(count),
            StrList(vec!["A".into(), "Ada L".into()]),
        ],
        2 => vec![
            Str("arbitrary subject".into()),
            Str("knows".into()),
            Str("arbitrary object".into()),
            I64(-100),
            I64(count),
            Str("node-r".into()),
            I64(0),
        ],
        _ => unreachable!(),
    };
    values
        .into_iter()
        .enumerate()
        .map(|(i, v)| (i as FieldRef, v))
        .collect()
}
pub fn update(domain: usize, key: RecordId, value: i64) -> TransactionOp {
    TransactionOp {
        id: key,
        field: UPDATE[domain],
        value: ScanValue::I64(value),
    }
}
