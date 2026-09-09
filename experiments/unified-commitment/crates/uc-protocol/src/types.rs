use crate::codec::{Reader, Wire};

pub const PROTOCOL_VERSION: u32 = 22;
pub const SESSION_READ_YOUR_WRITES: u32 = 1;
pub const SESSION_VALIDATE_ON_STAGE: u32 = 2;
pub const SESSION_SNAPSHOT_ISOLATION: u32 = 4;
pub const MAX_STAGED_OPS: usize = 4096;
pub const MAX_BATCH_OPS: usize = 4096;
pub const MAX_TRACKED_READS: usize = 4096;
pub type FieldRef = u16;
pub type Fields = Vec<(FieldRef, ScanValue)>;
pub type PageRow = (RecordId, Fields);
pub type ReadSet = Vec<(RecordId, FieldRef, ScanValue)>;

/// Raw UUID bytes in standard UUID byte order; no UUID-version restriction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RecordId(pub [u8; 16]);
impl RecordId {
    pub const fn from_u128(value: u128) -> Self {
        Self(value.to_be_bytes())
    }
    pub const fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }
}
impl Wire for RecordId {
    fn put(&self, out: &mut Vec<u8>) {
        16u64.put(out);
        out.extend_from_slice(&self.0);
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        if u64::read_wire(input)? != 16 {
            return Err("Malformed: UUID length must be 16".into());
        }
        Ok(Self(
            input.bytes(16)?.try_into().map_err(|_| "Malformed: UUID")?,
        ))
    }
}

// Tags are explicit protocol declarations, never inferred from Rust discriminants.
macro_rules! wire_struct {
    ($name:ident { $($field:ident : $ty:ty),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq)]
        pub struct $name { $(pub $field: $ty),* }
        impl Wire for $name {
            fn put(&self, out: &mut Vec<u8>) { $(self.$field.put(out);)* }
            fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> { Ok(Self { $($field: <$ty>::read_wire(input)?),* }) }
        }
    };
}
// Tuple enums use a separate macro so Rust macro hygiene binds each payload locally.
macro_rules! tuple_enum {
    ($name:ident { $($tag:literal => $variant:ident($ty:ty)),* $(,)? }) => {
        #[derive(Clone, Debug, PartialEq)]
        pub enum $name { $($variant($ty)),* }
        impl Wire for $name {
            fn put(&self, out: &mut Vec<u8>) {
                match self { $(Self::$variant(value) => { ($tag as u32).put(out); value.put(out); }),* }
            }
            fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
                match u32::read_wire(input)? { $($tag => Ok(Self::$variant(<$ty>::read_wire(input)?))),*, _ => Err(concat!("Malformed: unknown ", stringify!($name), " tag").into()) }
            }
        }
    }
}
macro_rules! unit_enum {
    ($name:ident { $($tag:literal => $variant:ident),* $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum $name { $($variant),* }
        impl Wire for $name {
            fn put(&self, out: &mut Vec<u8>) { match self { $(Self::$variant => ($tag as u32).put(out)),* } }
            fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
                match u32::read_wire(input)? { $($tag => Ok(Self::$variant)),*, _ => Err(concat!("Malformed: unknown ", stringify!($name), " tag").into()) }
            }
        }
    }
}

tuple_enum!(ScanValue { 0 => U32(u32), 1 => I64(i64), 2 => Bool(bool), 3 => Str(String), 4 => F64(f64), 5 => StrList(Vec<String>) });
unit_enum!(CompareOp { 0 => Eq, 1 => Ne, 2 => Lt, 3 => Le, 4 => Gt, 5 => Ge });
impl CompareOp {
    pub fn is_ordering(self) -> bool {
        !matches!(self, Self::Eq | Self::Ne)
    }
}
unit_enum!(AggregateFn { 0 => Count, 1 => Sum, 2 => Avg, 3 => Min, 4 => Max });
unit_enum!(ValueKind { 0 => U32, 1 => I64, 2 => Bool, 3 => Str, 4 => StrList });
unit_enum!(ErrorCode { 0 => UnknownField, 1 => Unsupported, 2 => Malformed, 3 => Unauthenticated, 4 => Unauthorized, 5 => RecordNotFound, 6 => NoSession, 7 => SessionOpen, 8 => SessionFull, 9 => Journal, 10 => Conflict, 11 => Duplicate, 12 => Storage, 13 => GuardFailed });
#[derive(Clone, Debug, PartialEq)]
pub enum Selection {
    All,
    Fields(Vec<FieldRef>),
}
impl Wire for Selection {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::All => {
                0u32.put(out);
            }
            Self::Fields(value) => {
                1u32.put(out);
                value.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::All),
            1 => Ok(Self::Fields(<Vec<FieldRef>>::read_wire(input)?)),
            _ => Err("Malformed: unknown Selection tag".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum JoinRelation {
    Neighbors(Option<String>),
    Parent,
    Children,
}
impl Wire for JoinRelation {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Neighbors(value) => {
                0u32.put(out);
                value.put(out);
            }
            Self::Parent => {
                1u32.put(out);
            }
            Self::Children => {
                2u32.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::Neighbors(<Option<String>>::read_wire(input)?)),
            1 => Ok(Self::Parent),
            2 => Ok(Self::Children),
            _ => Err("Malformed: unknown JoinRelation tag".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum ParentLookup {
    Parent(RecordId),
    NoParent,
    ChildNotFound,
}
impl Wire for ParentLookup {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Parent(value) => {
                0u32.put(out);
                value.put(out);
            }
            Self::NoParent => {
                1u32.put(out);
            }
            Self::ChildNotFound => {
                2u32.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::Parent(<RecordId>::read_wire(input)?)),
            1 => Ok(Self::NoParent),
            2 => Ok(Self::ChildNotFound),
            _ => Err("Malformed: unknown ParentLookup tag".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum WriteResult {
    Inserted,
    Duplicate,
    Replaced,
    NotFound,
    GuardFailed,
    Linked,
    AlreadyLinked,
    Deleted,
    Failed(ErrorCode),
}
impl Wire for WriteResult {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Inserted => {
                0u32.put(out);
            }
            Self::Duplicate => {
                1u32.put(out);
            }
            Self::Replaced => {
                2u32.put(out);
            }
            Self::NotFound => {
                3u32.put(out);
            }
            Self::GuardFailed => {
                4u32.put(out);
            }
            Self::Linked => {
                5u32.put(out);
            }
            Self::AlreadyLinked => {
                6u32.put(out);
            }
            Self::Deleted => {
                7u32.put(out);
            }
            Self::Failed(value) => {
                8u32.put(out);
                value.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::Inserted),
            1 => Ok(Self::Duplicate),
            2 => Ok(Self::Replaced),
            3 => Ok(Self::NotFound),
            4 => Ok(Self::GuardFailed),
            5 => Ok(Self::Linked),
            6 => Ok(Self::AlreadyLinked),
            7 => Ok(Self::Deleted),
            8 => Ok(Self::Failed(<ErrorCode>::read_wire(input)?)),
            _ => Err("Malformed: unknown WriteResult tag".into()),
        }
    }
}
wire_struct!(TransactionOp {
    id: RecordId,
    field: FieldRef,
    value: ScanValue
});
wire_struct!(Predicate {
    field: FieldRef,
    op: CompareOp,
    value: ScanValue
});
wire_struct!(AggregateSpec { func: AggregateFn, field: Option<FieldRef> });
wire_struct!(AggregateGroup { key: Fields, values: Vec<ScanValue> });
wire_struct!(JoinSpec { relation: JoinRelation, right_table: Option<String>, left: Selection, right: Selection, left_filter: Vec<Predicate>, right_filter: Vec<Predicate>, limit: Option<usize> });
wire_struct!(JoinedRow {
    left_id: RecordId,
    left: Fields,
    right_id: RecordId,
    right: Fields
});
wire_struct!(RelationDescriptor { name: String, kind: JoinRelation, target_table: Option<String> });
wire_struct!(FieldCapabilities {
    filter_eq: bool,
    scan: bool,
    update: bool
});
wire_struct!(FieldDescriptor {
    tag: FieldRef,
    name: String,
    value_kind: ValueKind,
    capabilities: FieldCapabilities
});
wire_struct!(RelationCapabilities {
    parent_children: bool,
    neighbors: bool
});
wire_struct!(DomainSchema { fields: Vec<FieldDescriptor>, relations: RelationCapabilities });
#[derive(Clone, Debug, PartialEq)]
pub enum WriteOp {
    Insert {
        id: RecordId,
        fields: Fields,
    },
    Replace {
        id: RecordId,
        fields: Fields,
    },
    ReplaceIf {
        id: RecordId,
        fields: Fields,
        guard: Predicate,
    },
    Delete {
        id: RecordId,
    },
    Link {
        left: RecordId,
        right: RecordId,
        relation: String,
    },
}
impl Wire for WriteOp {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Insert { id, fields } => {
                0u32.put(out);
                id.put(out);
                fields.put(out);
            }
            Self::Replace { id, fields } => {
                1u32.put(out);
                id.put(out);
                fields.put(out);
            }
            Self::ReplaceIf { id, fields, guard } => {
                2u32.put(out);
                id.put(out);
                fields.put(out);
                guard.put(out);
            }
            Self::Delete { id } => {
                3u32.put(out);
                id.put(out);
            }
            Self::Link {
                left,
                right,
                relation,
            } => {
                4u32.put(out);
                left.put(out);
                right.put(out);
                relation.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::Insert {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
            }),
            1 => Ok(Self::Replace {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
            }),
            2 => Ok(Self::ReplaceIf {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
                guard: <Predicate>::read_wire(input)?,
            }),
            3 => Ok(Self::Delete {
                id: <RecordId>::read_wire(input)?,
            }),
            4 => Ok(Self::Link {
                left: <RecordId>::read_wire(input)?,
                right: <RecordId>::read_wire(input)?,
                relation: <String>::read_wire(input)?,
            }),
            _ => Err("Malformed: unknown WriteOp tag".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Request {
    GetById {
        id: RecordId,
    },
    FilterEq {
        field: FieldRef,
        value: ScanValue,
    },
    ScanField {
        field: FieldRef,
    },
    UpdateField {
        id: RecordId,
        field: FieldRef,
        value: ScanValue,
    },
    Parent {
        id: RecordId,
    },
    Children {
        id: RecordId,
    },
    Neighbors {
        id: RecordId,
    },
    DescribeSchema,
    Authenticate {
        token: String,
    },
    Transaction {
        updates: Vec<TransactionOp>,
    },
    Hello {
        protocol_version: u32,
    },
    Begin,
    Commit,
    Rollback,
    BeginWith {
        flags: u32,
    },
    Query {
        select: Selection,
        filter: Vec<Predicate>,
        limit: Option<usize>,
    },
    Aggregate {
        group_by: Vec<FieldRef>,
        filter: Vec<Predicate>,
        aggregates: Vec<AggregateSpec>,
        limit: Option<usize>,
    },
    NeighborsByRelation {
        id: RecordId,
        relation: String,
    },
    ListRelationKinds,
    Join(JoinSpec),
    DescribeRelations,
    Insert {
        id: RecordId,
        fields: Fields,
    },
    Link {
        left: RecordId,
        right: RecordId,
        relation: String,
    },
    Replace {
        id: RecordId,
        fields: Fields,
    },
    Use {
        table: String,
    },
    ListTables,
    Delete {
        id: RecordId,
    },
    Compact,
    ReplaceIf {
        id: RecordId,
        fields: Fields,
        guard: Predicate,
    },
    Page {
        order_by: FieldRef,
        after: Option<(ScanValue, RecordId)>,
        limit: u64,
    },
    CountEdges {
        relation: String,
    },
    WriteBatch {
        ops: Vec<WriteOp>,
        atomic: bool,
    },
}
impl Wire for Request {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::GetById { id } => {
                0u32.put(out);
                id.put(out);
            }
            Self::FilterEq { field, value } => {
                1u32.put(out);
                field.put(out);
                value.put(out);
            }
            Self::ScanField { field } => {
                2u32.put(out);
                field.put(out);
            }
            Self::UpdateField { id, field, value } => {
                3u32.put(out);
                id.put(out);
                field.put(out);
                value.put(out);
            }
            Self::Parent { id } => {
                4u32.put(out);
                id.put(out);
            }
            Self::Children { id } => {
                5u32.put(out);
                id.put(out);
            }
            Self::Neighbors { id } => {
                6u32.put(out);
                id.put(out);
            }
            Self::DescribeSchema => {
                7u32.put(out);
            }
            Self::Authenticate { token } => {
                8u32.put(out);
                token.put(out);
            }
            Self::Transaction { updates } => {
                9u32.put(out);
                updates.put(out);
            }
            Self::Hello { protocol_version } => {
                10u32.put(out);
                protocol_version.put(out);
            }
            Self::Begin => {
                11u32.put(out);
            }
            Self::Commit => {
                12u32.put(out);
            }
            Self::Rollback => {
                13u32.put(out);
            }
            Self::BeginWith { flags } => {
                14u32.put(out);
                flags.put(out);
            }
            Self::Query {
                select,
                filter,
                limit,
            } => {
                15u32.put(out);
                select.put(out);
                filter.put(out);
                limit.put(out);
            }
            Self::Aggregate {
                group_by,
                filter,
                aggregates,
                limit,
            } => {
                16u32.put(out);
                group_by.put(out);
                filter.put(out);
                aggregates.put(out);
                limit.put(out);
            }
            Self::NeighborsByRelation { id, relation } => {
                17u32.put(out);
                id.put(out);
                relation.put(out);
            }
            Self::ListRelationKinds => {
                18u32.put(out);
            }
            Self::Join(value) => {
                19u32.put(out);
                value.put(out);
            }
            Self::DescribeRelations => {
                20u32.put(out);
            }
            Self::Insert { id, fields } => {
                21u32.put(out);
                id.put(out);
                fields.put(out);
            }
            Self::Link {
                left,
                right,
                relation,
            } => {
                22u32.put(out);
                left.put(out);
                right.put(out);
                relation.put(out);
            }
            Self::Replace { id, fields } => {
                23u32.put(out);
                id.put(out);
                fields.put(out);
            }
            Self::Use { table } => {
                24u32.put(out);
                table.put(out);
            }
            Self::ListTables => {
                25u32.put(out);
            }
            Self::Delete { id } => {
                26u32.put(out);
                id.put(out);
            }
            Self::Compact => {
                27u32.put(out);
            }
            Self::ReplaceIf { id, fields, guard } => {
                28u32.put(out);
                id.put(out);
                fields.put(out);
                guard.put(out);
            }
            Self::Page {
                order_by,
                after,
                limit,
            } => {
                29u32.put(out);
                order_by.put(out);
                after.put(out);
                limit.put(out);
            }
            Self::CountEdges { relation } => {
                30u32.put(out);
                relation.put(out);
            }
            Self::WriteBatch { ops, atomic } => {
                31u32.put(out);
                ops.put(out);
                atomic.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::GetById {
                id: <RecordId>::read_wire(input)?,
            }),
            1 => Ok(Self::FilterEq {
                field: <FieldRef>::read_wire(input)?,
                value: <ScanValue>::read_wire(input)?,
            }),
            2 => Ok(Self::ScanField {
                field: <FieldRef>::read_wire(input)?,
            }),
            3 => Ok(Self::UpdateField {
                id: <RecordId>::read_wire(input)?,
                field: <FieldRef>::read_wire(input)?,
                value: <ScanValue>::read_wire(input)?,
            }),
            4 => Ok(Self::Parent {
                id: <RecordId>::read_wire(input)?,
            }),
            5 => Ok(Self::Children {
                id: <RecordId>::read_wire(input)?,
            }),
            6 => Ok(Self::Neighbors {
                id: <RecordId>::read_wire(input)?,
            }),
            7 => Ok(Self::DescribeSchema),
            8 => Ok(Self::Authenticate {
                token: <String>::read_wire(input)?,
            }),
            9 => Ok(Self::Transaction {
                updates: <Vec<TransactionOp>>::read_wire(input)?,
            }),
            10 => Ok(Self::Hello {
                protocol_version: <u32>::read_wire(input)?,
            }),
            11 => Ok(Self::Begin),
            12 => Ok(Self::Commit),
            13 => Ok(Self::Rollback),
            14 => Ok(Self::BeginWith {
                flags: <u32>::read_wire(input)?,
            }),
            15 => Ok(Self::Query {
                select: <Selection>::read_wire(input)?,
                filter: <Vec<Predicate>>::read_wire(input)?,
                limit: <Option<usize>>::read_wire(input)?,
            }),
            16 => Ok(Self::Aggregate {
                group_by: <Vec<FieldRef>>::read_wire(input)?,
                filter: <Vec<Predicate>>::read_wire(input)?,
                aggregates: <Vec<AggregateSpec>>::read_wire(input)?,
                limit: <Option<usize>>::read_wire(input)?,
            }),
            17 => Ok(Self::NeighborsByRelation {
                id: <RecordId>::read_wire(input)?,
                relation: <String>::read_wire(input)?,
            }),
            18 => Ok(Self::ListRelationKinds),
            19 => Ok(Self::Join(<JoinSpec>::read_wire(input)?)),
            20 => Ok(Self::DescribeRelations),
            21 => Ok(Self::Insert {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
            }),
            22 => Ok(Self::Link {
                left: <RecordId>::read_wire(input)?,
                right: <RecordId>::read_wire(input)?,
                relation: <String>::read_wire(input)?,
            }),
            23 => Ok(Self::Replace {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
            }),
            24 => Ok(Self::Use {
                table: <String>::read_wire(input)?,
            }),
            25 => Ok(Self::ListTables),
            26 => Ok(Self::Delete {
                id: <RecordId>::read_wire(input)?,
            }),
            27 => Ok(Self::Compact),
            28 => Ok(Self::ReplaceIf {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
                guard: <Predicate>::read_wire(input)?,
            }),
            29 => Ok(Self::Page {
                order_by: <FieldRef>::read_wire(input)?,
                after: <Option<(ScanValue, RecordId)>>::read_wire(input)?,
                limit: <u64>::read_wire(input)?,
            }),
            30 => Ok(Self::CountEdges {
                relation: <String>::read_wire(input)?,
            }),
            31 => Ok(Self::WriteBatch {
                ops: <Vec<WriteOp>>::read_wire(input)?,
                atomic: <bool>::read_wire(input)?,
            }),
            _ => Err("Malformed: unknown Request tag".into()),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub enum Response {
    Record {
        id: RecordId,
        fields: Fields,
    },
    RecordList {
        records: Vec<RecordId>,
    },
    ScanValues {
        values: Vec<ScanValue>,
    },
    Id {
        id: RecordId,
    },
    Schema(DomainSchema),
    NotFound,
    NoParent,
    Ok,
    Err {
        code: ErrorCode,
        message: String,
    },
    TransactionFailed {
        index: usize,
        code: ErrorCode,
        message: String,
    },
    Hello {
        protocol_version: u32,
    },
    Staged {
        index: u32,
    },
    Rows {
        rows: Vec<PageRow>,
    },
    Groups {
        groups: Vec<AggregateGroup>,
    },
    RelationKinds {
        kinds: Vec<String>,
    },
    JoinedRows {
        rows: Vec<JoinedRow>,
    },
    Relations {
        relations: Vec<RelationDescriptor>,
    },
    Tables {
        names: Vec<String>,
        primary: String,
    },
    Compacted {
        records: u64,
        slots_reclaimed: u64,
        log_entries_folded: u64,
        edge_logs_folded: u64,
    },
    Count {
        count: u64,
    },
    BatchResults {
        results: Vec<WriteResult>,
    },
}
impl Wire for Response {
    fn put(&self, out: &mut Vec<u8>) {
        match self {
            Self::Record { id, fields } => {
                0u32.put(out);
                id.put(out);
                fields.put(out);
            }
            Self::RecordList { records } => {
                1u32.put(out);
                records.put(out);
            }
            Self::ScanValues { values } => {
                2u32.put(out);
                values.put(out);
            }
            Self::Id { id } => {
                3u32.put(out);
                id.put(out);
            }
            Self::Schema(value) => {
                4u32.put(out);
                value.put(out);
            }
            Self::NotFound => {
                5u32.put(out);
            }
            Self::NoParent => {
                6u32.put(out);
            }
            Self::Ok => {
                7u32.put(out);
            }
            Self::Err { code, message } => {
                8u32.put(out);
                code.put(out);
                message.put(out);
            }
            Self::TransactionFailed {
                index,
                code,
                message,
            } => {
                9u32.put(out);
                index.put(out);
                code.put(out);
                message.put(out);
            }
            Self::Hello { protocol_version } => {
                10u32.put(out);
                protocol_version.put(out);
            }
            Self::Staged { index } => {
                11u32.put(out);
                index.put(out);
            }
            Self::Rows { rows } => {
                12u32.put(out);
                rows.put(out);
            }
            Self::Groups { groups } => {
                13u32.put(out);
                groups.put(out);
            }
            Self::RelationKinds { kinds } => {
                14u32.put(out);
                kinds.put(out);
            }
            Self::JoinedRows { rows } => {
                15u32.put(out);
                rows.put(out);
            }
            Self::Relations { relations } => {
                16u32.put(out);
                relations.put(out);
            }
            Self::Tables { names, primary } => {
                17u32.put(out);
                names.put(out);
                primary.put(out);
            }
            Self::Compacted {
                records,
                slots_reclaimed,
                log_entries_folded,
                edge_logs_folded,
            } => {
                18u32.put(out);
                records.put(out);
                slots_reclaimed.put(out);
                log_entries_folded.put(out);
                edge_logs_folded.put(out);
            }
            Self::Count { count } => {
                19u32.put(out);
                count.put(out);
            }
            Self::BatchResults { results } => {
                20u32.put(out);
                results.put(out);
            }
        }
    }
    fn read_wire(input: &mut Reader<'_>) -> Result<Self, String> {
        match u32::read_wire(input)? {
            0 => Ok(Self::Record {
                id: <RecordId>::read_wire(input)?,
                fields: <Fields>::read_wire(input)?,
            }),
            1 => Ok(Self::RecordList {
                records: <Vec<RecordId>>::read_wire(input)?,
            }),
            2 => Ok(Self::ScanValues {
                values: <Vec<ScanValue>>::read_wire(input)?,
            }),
            3 => Ok(Self::Id {
                id: <RecordId>::read_wire(input)?,
            }),
            4 => Ok(Self::Schema(<DomainSchema>::read_wire(input)?)),
            5 => Ok(Self::NotFound),
            6 => Ok(Self::NoParent),
            7 => Ok(Self::Ok),
            8 => Ok(Self::Err {
                code: <ErrorCode>::read_wire(input)?,
                message: <String>::read_wire(input)?,
            }),
            9 => Ok(Self::TransactionFailed {
                index: <usize>::read_wire(input)?,
                code: <ErrorCode>::read_wire(input)?,
                message: <String>::read_wire(input)?,
            }),
            10 => Ok(Self::Hello {
                protocol_version: <u32>::read_wire(input)?,
            }),
            11 => Ok(Self::Staged {
                index: <u32>::read_wire(input)?,
            }),
            12 => Ok(Self::Rows {
                rows: <Vec<PageRow>>::read_wire(input)?,
            }),
            13 => Ok(Self::Groups {
                groups: <Vec<AggregateGroup>>::read_wire(input)?,
            }),
            14 => Ok(Self::RelationKinds {
                kinds: <Vec<String>>::read_wire(input)?,
            }),
            15 => Ok(Self::JoinedRows {
                rows: <Vec<JoinedRow>>::read_wire(input)?,
            }),
            16 => Ok(Self::Relations {
                relations: <Vec<RelationDescriptor>>::read_wire(input)?,
            }),
            17 => Ok(Self::Tables {
                names: <Vec<String>>::read_wire(input)?,
                primary: <String>::read_wire(input)?,
            }),
            18 => Ok(Self::Compacted {
                records: <u64>::read_wire(input)?,
                slots_reclaimed: <u64>::read_wire(input)?,
                log_entries_folded: <u64>::read_wire(input)?,
                edge_logs_folded: <u64>::read_wire(input)?,
            }),
            19 => Ok(Self::Count {
                count: <u64>::read_wire(input)?,
            }),
            20 => Ok(Self::BatchResults {
                results: <Vec<WriteResult>>::read_wire(input)?,
            }),
            _ => Err("Malformed: unknown Response tag".into()),
        }
    }
}
unit_enum!(InsertOutcome { 0 => Inserted, 1 => Duplicate });
unit_enum!(LinkOutcome { 0 => Linked, 1 => AlreadyLinked });
unit_enum!(ReplaceOutcome { 0 => Replaced, 1 => NotFound });
unit_enum!(DeleteOutcome { 0 => Deleted, 1 => NotFound });
unit_enum!(ReplaceIfOutcome { 0 => Replaced, 1 => NotFound, 2 => GuardFailed });
wire_struct!(CompactionReport {
    records: usize,
    slots_reclaimed: usize,
    log_entries_folded: usize,
    edge_logs_folded: usize
});
