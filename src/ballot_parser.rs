use std::borrow::Borrow;
use std::collections::hash_map::{Entry, HashMap};
use std::collections::HashSet;
use std::fmt::Display;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, stdin};
use std::result::Result;
use std::str::FromStr;
use vote::traits::Weight;

pub struct BallotParser<W> {
    pub candidates: Vec<String>,
    pub candidate_index: HashMap<String, usize>,
    pub ballots: Vec<(Box<[Box<[usize]>]>, W)>,
}

impl<W: FromStr + Weight> BallotParser<W>
    where W::Err: Display
{
    fn new() -> BallotParser<W> {
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
                let w = W::from_str(&line[..i].trim())
                    .map_err(|e| format!("cannot parse ballot weight: {}", e))?;
                if w <= W::zero() {
                    Err("non-positive ballot weight")?
                }
                (w, &line[i + 1..])
            }
            None => (W::one(), &line[..]),
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

pub fn parse_ballot_files<W, Str>(filenames: &[Str]) -> Result<BallotParser<W>, String>
    where W: FromStr + Weight,
          W::Err: Display,
          Str: Borrow<str>
{
    let mut bp = BallotParser::new();
    for filename in filenames {
        bp.add_ballot_file(filename.borrow())?;
    }
    Ok(bp)
}
