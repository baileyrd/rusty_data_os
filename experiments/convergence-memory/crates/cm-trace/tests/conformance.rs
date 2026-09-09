use cm_trace::{
    Model, Op, Value, answer_signature, column_digests, format, generate, hex, record_digests,
    results::Results, sha256,
};
const SMALL: &str = include_str!("fixtures/small.cmt");
#[test]
fn golden_trace_results_and_independent_python_sha256() {
    let trace = format::decode(SMALL).unwrap();
    assert_eq!(format::encode(&trace), SMALL);
    assert_eq!(trace, generate::small());
    let golden = include_str!("fixtures/small.results.cmt");
    let results = Results::decode(golden).unwrap();
    assert_eq!(results.encode(), golden);
    let mut model = Model::default();
    let actual: Vec<_> = trace
        .ops
        .iter()
        .map(|(_, op)| answer_signature(&model.apply(op)))
        .collect();
    assert_eq!(actual, results.operations);
    assert_eq!(record_digests(&model.records()), results.records);
    assert_eq!(column_digests(&model.records()), results.columns);
    let sums = include_str!("fixtures/SHA256SUMS");
    for (text, name) in [(SMALL, "small.cmt"), (golden, "small.results.cmt")] {
        assert!(
            sums.lines()
                .any(|l| l == format!("{}  {name}", hex(&sha256(text.as_bytes()))))
        );
    }
    assert_eq!(
        hex(&sha256(b"abc")),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
}
#[test]
fn small_hand_checked_semantics() {
    let trace = format::decode(SMALL).unwrap();
    assert_eq!(trace.ops.len(), 59);
    let mut model = Model::default();
    let answers: Vec<_> = trace.ops.iter().map(|(_, op)| model.apply(op)).collect();
    assert_eq!(answers[6].aggregate, Some((4, 0, Some(0), Some(0))));
    assert_eq!(
        answers[7].rows.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![generate::uuid(4), generate::uuid(8), generate::uuid(1)]
    );
    assert_eq!(
        answers[8].rows.iter().map(|m| m.id).collect::<Vec<_>>(),
        vec![generate::uuid(8), generate::uuid(1), generate::uuid(2)]
    );
    for (index, status) in [
        (9, "Duplicate"),
        (15, "Updated"),
        (21, "Replaced"),
        (27, "GuardFailed"),
        (33, "Replaced"),
        (39, "GuardFailed"),
        (41, "Replaced"),
        (43, "Deleted"),
        (44, "NotFound"),
        (49, "Inserted"),
        (55, "NotFound"),
        (56, "NotFound"),
        (57, "NotFound"),
        (58, "NotFound"),
    ] {
        assert_eq!(answers[index].outcome, status, "op {}", index + 1);
    }
    assert_eq!(answers[18].aggregate, Some((4, 17, Some(0), Some(17))));
    assert_eq!(answers[52].aggregate, Some((4, 3, Some(0), Some(3))));
    assert_eq!(
        answers[40].rows[0].fields[8],
        Value::Text("status-2".into())
    );
    assert_eq!(
        answers[42].rows[0].fields[8],
        Value::Text("status-text-guard".into())
    );
    assert_eq!(model.state[&generate::uuid(4)][10], Value::Int(3));
    assert_eq!(model.state.len(), 4);
    assert_eq!(
        Model::default().apply(&Op::Aggregate).aggregate,
        Some((0, 0, None, None))
    );
}
#[test]
fn malformed_and_noncanonical_inputs_fail_closed() {
    for bad in [
        SMALL.replace("CMT1\t1", "CMT1\t2"),
        SMALL.replacen("\n1\t", "\n2\t", 1),
        SMALL.replace("\t59\t", "\t58\t"),
        SMALL.replace("\t4\t59\t", "\t5\t59\t"),
        SMALL.replace("\tupdate\t", "\tunknown\t"),
        SMALL.replace("\t17\n", "\t-1\n"),
        SMALL.trim_end().to_string(),
    ] {
        assert!(format::decode(&bad).is_err());
    }
    for bad in ["b2", "i01", "i9223372036854775808", "sff", "l2:61", "sA0"] {
        assert!(format::parse_value(bad).is_err());
    }
    let result = include_str!("fixtures/small.results.cmt");
    assert!(Results::decode(&result.replace("column\t12", "column\t11")).is_err());
}
#[test]
fn seeded_1k_roundtrip_shape_and_every_identity_repeated() {
    let trace = generate::repeated(1000, 7);
    assert_eq!(format::decode(&format::encode(&trace)).unwrap(), trace);
    let mut inserts = std::collections::BTreeMap::new();
    for (_, op) in &trace.ops {
        if let Op::Insert(m) = op {
            *inserts.entry(m.id).or_insert(0) += 1;
            let Value::Text(content) = &m.fields[0] else {
                panic!()
            };
            assert!((200..=2000).contains(&content.len()));
            let Value::Text(metadata) = &m.fields[4] else {
                panic!()
            };
            assert!((50..=500).contains(&metadata.len()));
            let Value::Tags(tags) = &m.fields[2] else {
                panic!()
            };
            assert!(tags.len() <= 8);
        }
    }
    assert_eq!(
        trace
            .ops
            .iter()
            .filter(|(_, op)| matches!(
                op,
                Op::Guard {
                    field: 8,
                    equals: Value::Text(_),
                    ..
                }
            ))
            .count(),
        2000
    );
    assert_eq!(inserts.len(), 1000);
    assert!(inserts.values().all(|n| *n == 3));
    assert_ne!(trace, generate::repeated(1000, 8));
}

#[test]
fn streaming_observations_match_canonical_bytes_and_actual_record_counts() {
    use cm_trace::results::StreamingResults;
    let golden = Results::decode(include_str!("fixtures/small.results.cmt")).unwrap();
    for (index, mut result) in [golden.clone(), golden.clone(), golden.clone()]
        .into_iter()
        .enumerate()
    {
        if index == 1 {
            result.operations = vec!["large query signature\t".repeat(10000), "last".into()];
            result.records = (1..=100)
                .map(|id| (generate::uuid(id), "ab".repeat(32)))
                .collect();
            result.records.sort();
            result.durability = "engine declared UTF-8: \u{0394}\nline".into();
        } else if index == 2 {
            result.operations.clear();
            result.records.clear();
            result.columns = column_digests(&[]);
        }
        let path = cm_trace::run::test_directory("streamed-observations");
        let mut writer = StreamingResults::create(
            &path,
            result.input_sha256.clone(),
            result.engine.clone(),
            result.durability.clone(),
            result.operations.len(),
        )
        .unwrap();
        assert!(
            StreamingResults::create(
                &path,
                result.input_sha256.clone(),
                result.engine.clone(),
                result.durability.clone(),
                0
            )
            .is_err()
        );
        for (index, signature) in result.operations.iter().enumerate() {
            writer.operation(signature).unwrap();
            assert!(std::fs::read_to_string(&path).unwrap().ends_with(&format!(
                "op\t{}\t{}\n",
                index + 1,
                hex(signature.as_bytes())
            )));
        }
        assert!(writer.operation("extra").is_err());
        writer.finish(&result.records, &result.columns).unwrap();
        let encoded = std::fs::read_to_string(&path).unwrap();
        assert_eq!(encoded, result.encode());
        assert_eq!(Results::decode(&encoded).unwrap(), result);
        std::fs::remove_file(path).unwrap();
    }
    for invalid in ["", "ff", "0", "zz"] {
        let old = format!("durability\t{}\n", hex(golden.durability.as_bytes()));
        assert!(
            Results::decode(
                &golden
                    .encode()
                    .replace(&old, &format!("durability\t{invalid}\n"))
            )
            .is_err()
        );
    }
    let path = cm_trace::run::test_directory("incomplete-observations");
    let writer = StreamingResults::create(
        &path,
        golden.input_sha256,
        golden.engine,
        golden.durability,
        1,
    )
    .unwrap();
    assert!(writer.finish(&[], &column_digests(&[])).is_err());
    assert!(Results::decode(&std::fs::read_to_string(&path).unwrap()).is_err());
    std::fs::remove_file(path).unwrap();
}
