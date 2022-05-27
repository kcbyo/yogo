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
use unicase::UniCase;
use wait::Waiter;

// FIXME: We need to keep records around until they aren't found on the first page or for three
// days, whichever comes first. (Bug reported 26 May 2022.)

// Ok, yeah, I've seen the symptom, but the code is *supposed* to filter out anything that's older
// than some default number of days (which is equal to the number of days in the database). At
// least I think that's how it's meant to work, so what gives?

// If you peek into the history code, you'll see I've attempted to fix this by adding more slack.

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

    magnets.sort_unstable_by(|a, b| UniCase::new(&a.text).cmp(&UniCase::new(&b.text)));
    write_html(args, &magnets)?;
    history.write(args.take_after())?;

    Ok(())
}

fn write_html(args: &Args, magnets: &[Magnet]) -> io::Result<()> {
    static STYLE: &str = include_str!("../resource/style.css");

    let mut buf = String::new();
    writeln!(buf, "<style>\n{STYLE}\n</style>").expect("no way can this break");

    writeln!(buf, "<body>").unwrap();
    magnets
        .iter()
        .for_each(|magnet| format_line(&mut buf, magnet));
    writeln!(buf, "</body>").unwrap();

    match args.output() {
        Some(path) => fs::write(path, &buf),
        None => fs::write("listing.html", &buf),
    }
}

fn format_line(buf: &mut String, magnet: &Magnet) {
    let date = magnet.date;
    let size = &magnet.size;
    let link = &magnet.link;
    let text = &magnet.text;

    writeln!(
        buf,
        include_str!("../resource/template.html"),
        date = date,
        size = size,
        link = link,
        text = text,
    )
    .unwrap()
}
