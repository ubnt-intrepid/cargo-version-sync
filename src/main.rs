extern crate cargo_version_sync;
extern crate failure;
extern crate structopt;

use cargo_version_sync::runner::Runner;

mod cli {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    pub enum Cargo {
        #[structopt(name = "version-sync")]
        VersionSync(VersionSync),
    }

    #[derive(Debug, StructOpt)]
    pub struct VersionSync {
        /// Uses verbose output
        #[structopt(short = "v", long = "verbose")]
        pub verbose: bool,

        /// Do not overwrite the target files
        #[structopt(long = "check")]
        pub check: bool,
    }

    pub fn parse_args() -> VersionSync {
        match StructOpt::from_args() {
            Cargo::VersionSync(args) => args,
        }
    }
}

fn main() -> failure::Fallible<()> {
    let args = cli::parse_args();
    let runner = Runner::init()?;

    let diffs = runner.collect_diffs()?;
    if diffs.is_empty() {
        if args.verbose {
            println!("[cargo-version-sync] no change(s) detected.");
        }
        return Ok(());
    }

    if args.check {
        println!("[cargo-version-sync] detect unsynced files:");
        for diff in &diffs {
            runner.show_diff(&diff)?;
            println!();
        }
        std::process::exit(1);
    }

    for diff in &diffs {
        if args.verbose {
            println!(
                "[cargo-version-sync] update {}",
                runner.relative_file_path(diff)?.display()
            );
        }
        std::fs::write(&diff.file, &diff.replaced)?;
    }

    Ok(())
}
