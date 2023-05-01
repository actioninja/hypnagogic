#[macro_use]
mod util;

mod regressions {
    use super::*;
    use util::dir_tester::DirTester;

    test_dir!("basic_cut");
    test_dir!("simple_cuts");
    test_dir!("tall_cuts");
    test_dir!("tall_cuts_with_vis");
}
