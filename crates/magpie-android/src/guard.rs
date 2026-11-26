use std::panic::{AssertUnwindSafe, catch_unwind};

pub fn call<T>(fallback: T, body: impl FnOnce() -> T) -> T {
    catch_unwind(AssertUnwindSafe(body)).unwrap_or(fallback)
}
