mod args;
mod context;
mod history;
mod magnet;

use std::{fs, time::Duration};

use args::Args;
use context::Context;
use hashbrown::HashSet;
use history::History;

fn main() {
    let args = Args::parse();

    if let Err(e) = run(&args) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}

fn run(args: &Args) -> anyhow::Result<()> {
    let context = Context::new();
    let links = fs::read_to_string(&args.path)?;
    let urls = links.lines();

    let mut wait = false;
    let mut history = History::load()?;
    let mut magnets = Vec::new();
    let mut unique_magnet_filter = HashSet::new();

    for url in urls {
        if wait {
            std::thread::sleep(Duration::from_millis(750));
        }

        let mut recent = context.extract_recent(url, &mut unique_magnet_filter)?;
        recent.retain(|magnet| magnet.date >= args.take_after() && history.filter(magnet));
        magnets.extend(recent);
        wait = true;
    }

    // TODO: somehow format the magnet links to something usable?

    history.write(args.take_after())?;

    Ok(())
}
