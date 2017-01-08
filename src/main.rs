extern crate getopts;
#[cfg(feature = "use-gmp")]
extern crate gmp;
#[cfg(feature = "use-num-rational")]
extern crate num_rational;
extern crate vote;

mod ballot_parser;

use ballot_parser::parse_ballot_files;
use getopts::Options;
use std::env;
use std::fmt::Display;
use std::io::{Write, stderr};
use std::process::exit;
use std::str::FromStr;
use vote::schulze_stv::schulze_stv;
use vote::traits::{Weight, WeightOps};

const USAGE: &'static str = include_str!("usage.txt");

struct Calc {
    calc: &'static str,
    run: fn(&Calc, &str, usize, &[String]) -> Result<(), String>,
}

const CALCS: &'static [Calc] = &[
    #[cfg(feature = "use-gmp")]
    Calc { calc: "mpq", run: run::<gmp::mpq::Mpq> },
    #[cfg(feature = "use-num-rational")]
    Calc { calc: "num", run: run::<num_rational::BigRational> },
    Calc { calc: "hw", run: run::<vote::hw_float::HwFloat> },
];

fn main_result() -> Result<(), String> {
    let args = env::args().collect::<Vec<_>>();
    let program = &args[0];

    let mut opts = Options::new();
    opts.optopt("w",
                "winners",
                "elect an N-winner committee (default: 1)",
                "N");
    opts.optopt("",
                "calc",
                &format!("number type to use for calculations (default: {})",
                         CALCS[0].calc),
                "TYPE");
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

    let calc = match matches.opt_str("calc") {
        Some(calc_opt) => {
            CALCS.iter()
                .find(|calc| calc.calc == calc_opt)
                .ok_or_else(|| format!("unknown number type {}", calc_opt))
        }
        None => Ok(&CALCS[0]),
    }?;
    (calc.run)(calc, program, num_seats, &matches.free)
}

fn run<W>(calc: &Calc, program: &str, num_seats: usize, filenames: &[String]) -> Result<(), String>
    where W: Display + FromStr + Weight,
          W::Err: Display,
          for<'w> &'w W: WeightOps<W>
{
    let bp = parse_ballot_files::<W, _>(filenames)?;
    if bp.ballots.is_empty() {
        return Err(format!("{}: error: No ballots found", program));
    }

    println!("Tallying Schulze STV election (calc={}).", calc.calc);
    println!("");

    println!("Candidates ({}):", bp.candidates.len());
    let mut candidates_sorted = bp.candidates.clone();
    candidates_sorted.sort();
    for candidate in &candidates_sorted {
        println!("  {}", candidate);
    }
    println!("");

    let total_weight = bp.ballots.iter().fold(W::zero(), |acc, &(_, ref w)| acc + w);
    println!("Ballots ({}):", total_weight);
    for &(ref groups, ref w) in &bp.ballots {
        println!("  {}: {}",
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

    for set in &mut *winners {
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
    for set in &*winners {
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
