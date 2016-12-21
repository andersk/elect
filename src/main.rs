extern crate getopts;
extern crate gmp;
extern crate vote;

use getopts::Options;
use gmp::mpq::Mpq;
use gmp::mpz::Mpz;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::env;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write, stdin, stderr};
use std::process::exit;
use std::str::FromStr;
use vote::schulze_stv::schulze_stv;

const USAGE: &'static str = include_str!("usage.txt");

fn main() {
    let args = env::args().collect::<Vec<_>>();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("w",
                "winners",
                "elect an N-winner committee (default: 1)",
                "N");
    opts.optflag("", "help", "show this help message and exit");
    opts.optflag("", "version", "show the program version and exit");
    let matches = match opts.parse(&args[1..]) {
        Err(fail) => {
            writeln!(&mut stderr(), "{}: {}", program, fail).unwrap();
            exit(1)
        }
        Ok(matches) => matches,
    };

    if matches.opt_present("help") {
        print!("{}", opts.usage(USAGE));
        return;
    }

    if matches.opt_present("version") {
        println!("elect {}", env!("CARGO_PKG_VERSION"));
        return;
    }

    if matches.free.is_empty() {
        write!(&mut stderr(), "{}", opts.usage(USAGE)).unwrap();
        exit(1)
    }

    let num_seats = matches.opt_str("w").map(|s| s.parse().unwrap()).unwrap_or(1);

    let mut candidates = Vec::new();
    let mut candidate_index = HashMap::new();

    let ballots = matches.free
        .iter()
        .flat_map(|filename| {
            let file: Box<Read> = if filename == "-" {
                Box::new(stdin())
            } else {
                Box::new(File::open(filename).unwrap())
            };
            let buf = BufReader::new(file);
            buf.lines()
        })
        .map(|line| line.unwrap())
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            let (w, groups) = match line.find(':') {
                Some(i) => {
                    (Mpq::ratio(&Mpz::from_str(&line[..i]).unwrap(), &Mpz::one()), &line[i + 1..])
                }
                None => (Mpq::one(), &line[..]),
            };
            (groups.split('>')
                 .map(|group| {
                    group.split('=')
                        .map(|name| {
                            let name = name.trim();
                            match candidate_index.entry(name.to_string()) {
                                Entry::Occupied(e) => *e.get(),
                                Entry::Vacant(e) => {
                                    let n = candidates.len();
                                    e.insert(n);
                                    candidates.push(name.to_string());
                                    n
                                }
                            }
                        })
                        .collect::<Vec<_>>()
                        .into_boxed_slice()
                })
                 .collect::<Vec<_>>()
                 .into_boxed_slice(),
             w)
        })
        .collect::<Vec<_>>()
        .into_boxed_slice();

    if ballots.is_empty() {
        writeln!(&mut stderr(), "error: No ballots found").unwrap();
        exit(1);
    }

    println!("Tallying Schulze STV election.");
    println!("");

    println!("Candidates ({}):", candidates.len());
    let mut candidates_sorted = candidates.clone();
    candidates_sorted.sort();
    for candidate in &candidates_sorted {
        println!("  {}", candidate);
    }
    println!("");

    let total_weight = ballots.iter().map(|&(_, ref w)| w).fold(Mpq::zero(), |acc, x| acc + x);
    println!("Ballots ({:?}):", total_weight);
    for &(ref groups, ref w) in ballots.iter() {
        println!("  {:?}: {}",
                 w,
                 groups.iter()
                     .map(|group| {
                         group.iter().map(|&c| &candidates[c][..]).collect::<Vec<_>>().join(" = ")
                     })
                     .collect::<Vec<_>>()
                     .join(" > "));
    }
    println!("");

    let mut winners = schulze_stv(candidates.len(), num_seats, &ballots);

    for set in winners.iter_mut() {
        set.sort_by(|&a, &b| candidates[a].cmp(&candidates[b]));
    }
    winners.sort_by(|a, b| {
        a.iter().map(|&i| &candidates[i]).cmp(b.iter().map(|&i| &candidates[i]))
    });

    let set_suffix = if num_seats == 1 { "" } else { " set" };
    if winners.len() == 1 {
        println!("Winner{}:", set_suffix);
    } else {
        println!("Tied winner{}s:", set_suffix);
    }
    for set in winners.iter() {
        println!("  {}",
                 set.iter().map(|&c| &candidates[c][..]).collect::<Vec<_>>().join(", "));
    }
}
