use crate::util::deep_dir_compare::deep_compare_path;
use crate::util::run::run_with_args;
use std::fs::read_to_string;

pub struct DirTester {
    dir: std::path::PathBuf,
}

impl DirTester {
    pub fn new(dir: &std::path::Path) -> Self {
        Self {
            dir: dir.to_path_buf(),
        }
    }

    pub fn run(&mut self) {
        let args_path = self.dir.join("args.txt");
        let args = read_to_string(args_path).unwrap();
        let mut args: Vec<String> = args.lines().map(|s| s.to_string()).collect();

        args.push("--output".to_string());
        let out_dir = self.dir.join("actual-OUTPUT");
        args.push(out_dir.to_str().unwrap().to_string());
        args.push("input".to_string());

        let mut command = run_with_args(args).unwrap();
        command.current_dir(self.dir.clone());
        let _ = command.output().unwrap();

        let expected_path = self.dir.join("expected");

        let res = deep_compare_path(&expected_path, &out_dir);

        if let Err(res) = res {
            panic!("Deep compare failed: {res:?}");
        }
    }
}

#[macro_export]
macro_rules! test_dir {
    ($dir:literal) => {
        ::paste::paste! {
            #[test]
            fn [<dir_ $dir>]() {
                let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/", "test_files/", $dir);
                let dir = std::path::Path::new(dir);
                let mut tester = DirTester::new(dir);
                tester.run();
            }
        }
    };
}
