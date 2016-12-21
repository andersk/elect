extern crate getopts;
extern crate gmp;
extern crate vote;

mod ballot_parser;

use ballot_parser::parse_ballot_files;
use getopts::Options;
use gmp::mpq::Mpq;
use std::env;
use std::io::{Write, stderr};
use std::process::exit;
use vote::schulze_stv::schulze_stv;

const USAGE: &'static str = include_str!("usage.txt");

fn main_result() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("w",
                "winners",
                "elect an N-winner committee (default: 1)",
                "N");
    opts.optflag("", "help", "show this help message and exit");
    opts.optflag("", "version", "show the program version and exit");
    let matches = opts.parse(&args[1..]).map_err(|e| format!("{}: error: {}", program, e))?;

    if matches.opt_present("help") {
        print!("{}", opts.usage(USAGE));
        return Ok(());
    }

    if matches.opt_present("version") {
        println!("elect {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if matches.free.is_empty() {
        write!(&mut stderr(), "{}", opts.usage(USAGE)).expect("failed printing to stderr");
        exit(1)
    }

    let num_seats = matches.opt_str("w")
        .map(|s| s.parse().map_err(|e| format!("{}: error: -w argument: {}", program, e)))
        .unwrap_or(Ok(1))?;

    let bp = parse_ballot_files(&matches.free)?;
    if bp.ballots.is_empty() {
        return Err(format!("{}: error: No ballots found", program));
    }

    println!("Tallying Schulze STV election.");
    println!("");

    println!("Candidates ({}):", bp.candidates.len());
    let mut candidates_sorted = bp.candidates.clone();
    candidates_sorted.sort();
    for candidate in &candidates_sorted {
        println!("  {}", candidate);
    }
    println!("");

    let total_weight = bp.ballots.iter().map(|&(_, ref w)| w).fold(Mpq::zero(), |acc, x| acc + x);
    println!("Ballots ({:?}):", total_weight);
    for &(ref groups, ref w) in bp.ballots.iter() {
        println!("  {:?}: {}",
                 w,
                 groups.iter()
                     .map(|group| {
                         group.iter()
                             .map(|&c| &bp.candidates[c][..])
                             .collect::<Vec<_>>()
                             .join(" = ")
                     })
                     .collect::<Vec<_>>()
                     .join(" > "));
    }
    println!("");

    let mut winners = schulze_stv(bp.candidates.len(), num_seats, &bp.ballots);

    for set in winners.iter_mut() {
        set.sort_by(|&a, &b| bp.candidates[a].cmp(&bp.candidates[b]));
    }
    winners.sort_by(|a, b| {
        a.iter().map(|&i| &bp.candidates[i]).cmp(b.iter().map(|&i| &bp.candidates[i]))
    });

    let set_suffix = if num_seats == 1 { "" } else { " set" };
    if winners.len() == 1 {
        println!("Winner{}:", set_suffix);
    } else {
        println!("Tied winner{}s:", set_suffix);
    }
    for set in winners.iter() {
        println!("  {}",
                 set.iter().map(|&c| &bp.candidates[c][..]).collect::<Vec<_>>().join(", "));
    }
    Ok(())
}

fn main() {
    main_result().unwrap_or_else(|e| {
        writeln!(&mut stderr(), "{}", e).expect("failed printing to stderr");
        exit(1)
    })
}
