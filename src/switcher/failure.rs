use crate::error::Result;

#[cfg(test)]
use crate::error::VexError;
#[cfg(test)]
use std::cell::RefCell;

#[cfg(test)]
#[derive(Debug, Clone)]
pub(super) enum TestFailurePoint {
    BinLink(String),
}

#[cfg(test)]
thread_local! {
    static TEST_FAILURE_POINT: RefCell<Option<TestFailurePoint>> = const { RefCell::new(None) };
}

#[cfg(test)]
pub(super) struct TestFailureGuard;

#[cfg(test)]
impl Drop for TestFailureGuard {
    fn drop(&mut self) {
        TEST_FAILURE_POINT.with(|failure| {
            failure.borrow_mut().take();
        });
    }
}

#[cfg(test)]
pub(super) fn inject_test_failure(point: TestFailurePoint) -> TestFailureGuard {
    TEST_FAILURE_POINT.with(|failure| {
        *failure.borrow_mut() = Some(point);
    });
    TestFailureGuard
}

#[cfg(test)]
pub(super) fn maybe_fail_bin_link(bin_name: &str) -> Result<()> {
    let should_fail = TEST_FAILURE_POINT.with(|failure| {
        matches!(
            failure.borrow().as_ref(),
            Some(TestFailurePoint::BinLink(name)) if name == bin_name
        )
    });
    if should_fail {
        TEST_FAILURE_POINT.with(|failure| {
            failure.borrow_mut().take();
        });
        return Err(VexError::Config(format!(
            "Injected test failure while linking {}",
            bin_name
        )));
    }
    Ok(())
}

#[cfg(not(test))]
pub(super) fn maybe_fail_bin_link(_bin_name: &str) -> Result<()> {
    Ok(())
}
