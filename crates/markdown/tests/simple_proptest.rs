use proptest::prelude::*;

proptest! {
    #[test]
    fn test_simple(s in ".*") {
        // A string's byte length is always at least its char count; a trivial
        // but non-vacuous invariant (the old `len() >= 0` was always true).
        prop_assert!(s.len() >= s.chars().count());
    }
}
