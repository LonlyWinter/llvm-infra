use llvm_infra::meta::IDGenerater;

#[test]
fn test_id() {
    let mut id = IDGenerater::default();
    let name1 = id.generate_const("12".to_string());
    {
        let name2 = name1.clone();
        assert_eq!(name2.count(), 3);
    }
    assert_eq!(name1.count(), 2);
}
