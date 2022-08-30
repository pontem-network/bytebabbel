use anyhow::{anyhow, Result};
use libtest_mimic::{Arguments, Outcome, Test};

mod testssol;

use crate::testssol::clog::{log_init, CustLogger};
use crate::testssol::STest;

fn run_test(test: &STest) -> Result<()> {
    test.run().map_err(|err| {
        log::error!("{err:?}");
        anyhow!("{}", CustLogger::flush_and_get())
    })
}

fn main() {
    log_init();

    let mut tests = STest::from_sol_dir()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(index, data)| {
            let name = data.test_name(index);
            let testfn: Box<dyn Fn() -> Outcome + Send + Sync> =
                Box::new(move || match run_test(&data) {
                    Ok(()) => Outcome::Passed,
                    Err(err) => Outcome::Failed {
                        msg: Some(format!("{}", err)),
                    },
                });
            let is_ignored = name.contains("ignore");
            Ok(Test {
                name,
                kind: String::new(),
                is_ignored,
                is_bench: false,
                data: testfn,
            })
        })
        .collect::<Result<Vec<_>>>()
        .unwrap();
    tests.sort_unstable_by(|a, b| a.name.cmp(&b.name));

    let configs = Arguments::from_args();

    // Build logs
    let error = CustLogger::flush_and_get();
    if !error.is_empty() {
        println!("{error}");
    }

    // run tests
    libtest_mimic::run_tests(&configs, tests, |test| (test.data)()).exit()
}
