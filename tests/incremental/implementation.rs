use crate::constants;
use crate::counter::Counter;
use crate::log::Log;
use crate::memoized_dep_inputs;
use crate::memoized_inputs;
use crate::memoized_volatile;

use gluon_salsa as salsa;

pub(crate) trait TestContext: salsa::Database {
    fn clock(&self) -> &Counter;
    fn log(&self) -> &Log;
}

#[salsa::database(
    constants::Constants,
    memoized_dep_inputs::MemoizedDepInputs,
    memoized_inputs::MemoizedInputs,
    memoized_volatile::MemoizedVolatile
)]
#[derive(Default)]
pub(crate) struct TestContextImpl {
    runtime: salsa::Runtime<TestContextImpl>,
    clock: Counter,
    log: Log,
}

impl TestContextImpl {
    pub(crate) fn assert_log(&self, expected_log: &[&str]) {
        let expected_text = &format!("{:#?}", expected_log);
        let actual_text = &format!("{:#?}", self.log().take());

        if expected_text == actual_text {
            return;
        }

        for diff in diff::lines(expected_text, actual_text) {
            match diff {
                diff::Result::Left(l) => println!("-{}", l),
                diff::Result::Both(l, _) => println!(" {}", l),
                diff::Result::Right(r) => println!("+{}", r),
            }
        }

        panic!("incorrect log results");
    }
}

impl TestContext for TestContextImpl {
    fn clock(&self) -> &Counter {
        &self.clock
    }

    fn log(&self) -> &Log {
        &self.log
    }
}

impl salsa::Database for TestContextImpl {
    fn salsa_runtime(&self) -> &salsa::Runtime<Self> {
        &self.runtime
    }

    fn salsa_runtime_mut(&mut self) -> &mut salsa::Runtime<Self> {
        &mut self.runtime
    }
}
