use anyhow::{anyhow, Result};
use libtest_mimic::{Arguments, Outcome, Test};

use test_infra::{init_log, init_log_with_buff_and_name, CustLogger};

use crate::testssol::STest;

mod testssol;

fn run_test(name: &str, test: &STest) -> Result<()> {
    init_log_with_buff_and_name(name);

    test.run().map_err(|err| {
        log::error!("{err:?}");
        anyhow!("{}", CustLogger::flush_and_get())
    })
}

fn main() {
    init_log();

    let mut tests = STest::from_sol_dir()
        .unwrap()
        .into_iter()
        .enumerate()
        .map(|(index, data)| {
            let name = data.test_name(index);
            let run_name = name.clone();

            let testfn: Box<dyn Fn() -> Outcome + Send + Sync> =
                Box::new(move || match run_test(&run_name, &data) {
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

    // run tests
    libtest_mimic::run_tests(&configs, tests, |test| (test.data)()).exit()
}
