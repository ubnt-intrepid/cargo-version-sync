extern crate cargo_version_sync;
extern crate difference;
extern crate failure;
extern crate structopt;

mod cli {
    use structopt::StructOpt;

    #[derive(Debug, StructOpt)]
    pub enum Cargo {
        #[structopt(name = "version-sync")]
        VersionSync(VersionSync),
    }

    #[derive(Debug, StructOpt)]
    pub struct VersionSync {
        /// User verbose output
        #[structopt(short = "v", long = "verbose")]
        pub verbose: bool,

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

use cargo_version_sync::Runner;
use difference::Changeset;

fn main() -> failure::Fallible<()> {
    let args = cli::parse_args();
    let verbose = args.dry_run || args.verbose;
    let overwrite = !args.check && !args.dry_run;

    let changeset = Runner::init()?.run()?;
    if changeset.is_empty() {
        return Ok(());
    }

    if args.check {
        std::process::exit(1);
    }

    if verbose {
        println!("change(s) are detected:");
    }
    for change in changeset {
        if verbose {
            println!("in {}", change.file.display());
            println!(
                "{}",
                Changeset::new(&change.content, &change.replaced, "\n")
            );
        }
        if overwrite {
            change.dump()?;
        }
    }

    Ok(())
}
