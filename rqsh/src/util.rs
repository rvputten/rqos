use std::collections::HashSet;

pub fn dedup_preserve_order(job_names: Vec<String>) -> Vec<String> {
    let mut seen = HashSet::new();
    job_names
        .into_iter()
        .filter(|job| seen.insert(job.clone()))
        .collect()
}

#[test]
fn test_dedup_preserve_order() {
    let jobs = vec![
        "a".to_string(),
        "f".to_string(),
        "e".to_string(),
        "b".to_string(),
        "a".to_string(),
        "c".to_string(),
        "b".to_string(),
        "d".to_string(),
    ];
    let expected = vec![
        "a".to_string(),
        "f".to_string(),
        "e".to_string(),
        "b".to_string(),
        "c".to_string(),
        "d".to_string(),
    ];
    assert_eq!(dedup_preserve_order(jobs), expected);
}
