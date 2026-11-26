use magpie_core::guard;

const FALLBACK: i32 = -1;

#[test]
fn panic_returns_fallback_instead_of_unwinding() {
    let result = guard::call(FALLBACK, || panic!("engine failed"));
    assert_eq!(result, FALLBACK);
}

#[test]
fn success_returns_body_value() {
    assert_eq!(guard::call(FALLBACK, || 7), 7);
}
