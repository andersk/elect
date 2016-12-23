use gmp::mpq::Mpq;
use gmp::mpz::Mpz;
use std::borrow::Borrow;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, stdin};
use std::result::Result;
use std::str::FromStr;

pub struct BallotParser {
    pub candidates: Vec<String>,
    pub candidate_index: HashMap<String, usize>,
    pub ballots: Vec<(Box<[Box<[usize]>]>, Mpq)>,
}

fn parse_rational(s: &str) -> Result<Mpq, ()> {
    match s.find('/') {
        Some(i) => Ok(Mpq::ratio(&Mpz::from_str(&s[..i])?, &Mpz::from_str(&s[i + 1..])?)),
        None => Ok(Mpq::ratio(&Mpz::from_str(s)?, &Mpz::one())),
    }
}

impl BallotParser {
    fn new() -> BallotParser {
        BallotParser {
            candidates: Vec::new(),
            candidate_index: HashMap::new(),
            ballots: Vec::new(),
        }
    }

    fn parse_candidate(&mut self, name: &str, used: &mut HashSet<usize>) -> Result<usize, String> {
        let name = name.trim();
        if name.is_empty() {
            Err("empty candidate name")?
        }
        let n = match self.candidate_index.entry(name.to_string()) {
            Entry::Occupied(e) => *e.get(),
            Entry::Vacant(e) => {
                let n = self.candidates.len();
                e.insert(n);
                self.candidates.push(name.to_string());
                n
            }
        };
        if used.insert(n) {
            Ok(n)
        } else {
            Err(format!("candidate repeated: {}", name))
        }
    }

    fn parse_group(&mut self,
                   group: &str,
                   used: &mut HashSet<usize>)
                   -> Result<Box<[usize]>, String> {
        Ok(group.split('=')
            .map(|name| self.parse_candidate(name, used))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice())
    }

    fn parse_groups(&mut self,
                    groups: &str,
                    used: &mut HashSet<usize>)
                    -> Result<Box<[Box<[usize]>]>, String> {
        Ok(groups.split('>')
            .map(|group| self.parse_group(group, used))
            .collect::<Result<Vec<_>, _>>()?
            .into_boxed_slice())
    }

    fn add_ballot(&mut self, line: &str) -> Result<(), String> {
        let line = line.trim();
        if line.is_empty() {
            return Ok(());
        }

        let (w, groups) = match line.find(':') {
            Some(i) => {
                let w = parse_rational(&line[..i]).map_err(|()| "cannot parse ballot weight")?;
                if w <= Mpq::zero() {
                    Err("non-positive ballot weight")?
                }
                (w, &line[i + 1..])
            }
            None => (Mpq::one(), &line[..]),
        };

        let ballot = (self.parse_groups(groups, &mut HashSet::new())?, w);
        self.ballots.push(ballot);
        Ok(())
    }

    fn add_ballots<R: Read>(&mut self, buf: BufReader<R>) -> Result<(), (usize, String)> {
        for (lineno, line) in buf.lines().enumerate() {
            self.add_ballot(&line.map_err(|e| (lineno, e.to_string()))?)
                .map_err(|e| (lineno, e))?;
        }
        Ok(())
    }

    fn add_ballot_file(&mut self, filename: &str) -> Result<(), String> {
        let file: Box<Read> = if filename == "-" {
            Box::new(stdin())
        } else {
            Box::new(File::open(filename).map_err(|e| format!("error: {}: {}", filename, e))?)
        };
        self.add_ballots(BufReader::new(file))
            .map_err(|(lineno, e)| format!("{}:{}: error: {}", filename, lineno + 1, e))
    }
}

pub fn parse_ballot_files<Str>(filenames: &[Str]) -> Result<BallotParser, String>
    where Str: Borrow<str>
{
    let mut bp = BallotParser::new();
    for filename in filenames.iter() {
        bp.add_ballot_file(filename.borrow())?;
    }
    Ok(bp)
}
