mod args;
mod context;
mod history;
mod magnet;
mod wait;

use std::{fmt::Write, fs, io};

use args::Args;
use context::Context;
use hashbrown::HashSet;
use history::History;
use magnet::Magnet;
use wait::Waiter;

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

    let mut waiter = Waiter::new();
    let mut history = History::load()?;
    let mut magnets = Vec::new();
    let mut unique_magnet_filter = HashSet::new();

    for url in links.lines() {
        waiter.wait();

        let mut recent = context.extract_recent(url, &mut unique_magnet_filter)?;
        recent.retain(|magnet| magnet.date >= args.take_after() && history.filter(magnet));
        magnets.extend(recent);
    }

    write_html(&magnets)?;
    history.write(args.take_after())?;

    Ok(())
}

fn write_html(magnets: &[Magnet]) -> io::Result<()> {
    let mut buf = String::new();
    buf += "<ul>\n";
    magnets
        .iter()
        .for_each(|magnet| format_line(&mut buf, magnet));
    buf += "</ul>\n";
    fs::write("listing.html", &buf)
}

fn format_line(buf: &mut String, magnet: &Magnet) {
    let date = magnet.date;
    let size = &magnet.size;
    let link = &magnet.link;
    let text = &magnet.text;
    writeln!(
        buf,
        r#"  <li><strong>{date} {size}</strong> <a href="{link}">{text}</a></li>"#
    )
    .expect("pretty sure this can't break")
}
