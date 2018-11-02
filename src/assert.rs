use crate::runner::Runner;

pub fn assert_version_sync() {
    let runner = Runner::init().expect("failed to initialize the runner");
    match runner.collect_changeset() {
        Ok(changeset) => {
            if !changeset.diffs.is_empty() {
                eprintln!("{}", changeset);
                panic!();
            }
        }
        Err(e) => panic!("error during checking version sync: {}", e),
    }
}
