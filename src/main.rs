extern crate cargo_version_sync;
extern crate failure;
extern crate structopt;

use cargo_version_sync::Runner;

mod cli {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    pub enum Cargo {
        #[structopt(name = "version-sync")]
        VersionSync(VersionSync),
    }

    #[derive(Debug, StructOpt)]
    pub struct VersionSync {
        /// Do not overwrite the changeset to files
        #[structopt(short = "n", long = "dry-run")]
        pub dry_run: bool,

        /// Do not overwrite the changeset to files,
        /// and return non-zero exit code when the version number(s) are not synced
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
        println!("[cargo-version-sync] no change(s) detected.");
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

    if args.dry_run {
        println!("[cargo-version-sync] The following file(s) will be updated:");
        for diff in &diffs {
            println!("  - {}", runner.relative_file_path(diff)?.display());
        }
    } else {
        for diff in &diffs {
            std::fs::write(&diff.file, &diff.replaced)?;
        }
    }

    Ok(())
}
