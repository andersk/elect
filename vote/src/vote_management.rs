use gmp::mpq::Mpq;
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::vec::Vec;

struct BallotState {
    visited: bool,
    prev: usize,
    edge_flow: Box<[Mpq]>,
    sink_flow: Mpq,
    count: i32,
}

#[derive(Clone)]
struct CandidateState {
    prev: usize,
    count: i32,
}

pub fn strength<Ballot>(num_candidates: usize, ballots: &[(Ballot, Mpq)]) -> Mpq
    where Ballot: Borrow<[bool]>
{
    let candidate_ballots = &mut vec![Vec::new(); num_candidates][..];
    for (b, &(ref cs, _)) in ballots.iter().enumerate() {
        for (c, &s) in cs.borrow().iter().enumerate() {
            if s {
                candidate_ballots[c].push(b);
            }
        }
    }
    let ballot_states = &mut ballots.iter()
        .map(|&(_, ref w)| {
            BallotState {
                visited: false,
                prev: !0,
                edge_flow: vec![Mpq::zero(); num_candidates].into_boxed_slice(),
                sink_flow: w.clone(),
                count: 0,
            }
        })
        .collect::<Vec<_>>()[..];
    let candidate_states = &mut vec![CandidateState {
        prev: !0,
        count: 0,
    }; num_candidates][..];

    let mut total_flow = Mpq::zero();
    let mut queue = VecDeque::new();

    loop {
        for (b, bs) in ballot_states.iter_mut().enumerate() {
            if !bs.sink_flow.is_zero() {
                bs.visited = true;
                queue.push_back(b);
            }
        }

        let mut found = Vec::new();
        'search: loop {
            match queue.pop_front() {
                None => {
                    return total_flow;
                }
                Some(b) => {
                    let (ref cs, _) = ballots[b];
                    for (c, &s) in cs.borrow().iter().enumerate() {
                        if !s || candidate_states[c].prev != !0 {
                            continue;
                        }
                        candidate_states[c].prev = b;
                        found.push(c);
                        if found.len() == num_candidates {
                            break 'search;
                        }
                        for &b1 in &candidate_ballots[c] {
                            if ballot_states[b1].edge_flow[c].is_zero() ||
                               ballot_states[b1].visited {
                                continue;
                            }
                            ballot_states[b1].visited = true;
                            ballot_states[b1].prev = c;
                            queue.push_back(b1);
                        }
                    }
                }
            }
        }

        let mut sunk = 0;
        for &c in found.iter().rev() {
            let b = candidate_states[c].prev;
            let count = candidate_states[c].count + 1;
            ballot_states[b].count += count;
            let c1 = ballot_states[b].prev;
            if c1 == !0 {
                sunk += count;
            } else {
                candidate_states[c1].count += count;
            }
        }
        debug_assert_eq!(sunk, num_candidates as i32);

        let flow = found.iter()
            .map(|&c| {
                let b = candidate_states[c].prev;
                let c1 = ballot_states[b].prev;
                (if c1 == !0 {
                    &ballot_states[b].sink_flow
                } else {
                    &ballot_states[b].edge_flow[c1]
                } / Mpq::from(ballot_states[b].count as i64))
            })
            .min()
            .unwrap();
        debug_assert!(!flow.is_zero());
        total_flow = total_flow + &flow;

        for c in found {
            let b = candidate_states[c].prev;
            let c1 = ballot_states[b].prev;
            ballot_states[b].edge_flow[c] = &ballot_states[b].edge_flow[c] +
                                            &flow * Mpq::from(candidate_states[c].count as i64 + 1);
            if ballot_states[b].count != 0 {
                let edge_flow = if c1 == !0 {
                    &mut ballot_states[b].sink_flow
                } else {
                    &mut ballot_states[b].edge_flow[c1]
                };
                *edge_flow = &*edge_flow - &flow * Mpq::from(ballot_states[b].count as i64);
                ballot_states[b].count = 0;
            }

            candidate_states[c].prev = !0;
            candidate_states[c].count = 0;
        }

        for bs in &mut *ballot_states {
            bs.visited = false;
            bs.prev = !0;
            debug_assert_eq!(bs.count, 0);
        }
        queue.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::strength;
    use gmp::mpq::Mpq;

    const Q: fn(i64) -> Mpq = Mpq::from;

    #[test]
    fn test_strength_1() {
        // Wikipedia
        let ballots: &[(&[bool], Mpq)] = &[(&[true, false], Q(12)),
                                           (&[false, true], Q(0)),
                                           (&[true, true], Q(51)),
                                           (&[false, false], Q(27))];
        assert_eq!(strength(2, ballots), Q(63) / Q(2));
    }

    #[test]
    fn test_strength_2() {
        // Wikipedia
        let ballots: &[(&[bool], Mpq)] =
            &[(&[true, false], Q(38)), (&[false, true], Q(27)), (&[true, true], Q(12))];
        assert_eq!(strength(2, ballots), Q(77) / Q(2));
    }

    #[test]
    fn test_strength_3() {
        // Schulze’s calcul02.pdf
        let ballots: &[(&[bool], Mpq)] =
            &[(&[true, true, true, true], Q(36_597383) / Q(1_000000)),
              (&[true, true, true, false], Q(5_481150) / Q(1_000000)),
              (&[true, true, false, true], Q(13_279131) / Q(1_000000)),
              (&[true, true, false, false], Q(4_859413) / Q(1_000000)),
              (&[true, false, true, true], Q(35_425375) / Q(1_000000)),
              (&[true, false, true, false], Q(5_490934) / Q(1_000000)),
              (&[true, false, false, true], Q(22_855333) / Q(1_000000)),
              (&[true, false, false, false], Q(19_835570) / Q(1_000000)),
              (&[false, true, true, true], Q(22_928716) / Q(1_000000)),
              (&[false, true, true, false], Q(5_538309) / Q(1_000000)),
              (&[false, true, false, true], Q(13_130227) / Q(1_000000)),
              (&[false, true, false, false], Q(6_056291) / Q(1_000000)),
              (&[false, false, true, true], Q(23_992772) / Q(1_000000)),
              (&[false, false, true, false], Q(16_699207) / Q(1_000000)),
              (&[false, false, false, true], Q(98_165759) / Q(1_000000)),
              (&[false, false, false, false], Q(129_664430) / Q(1_000000))];
        assert_eq!(strength(4, ballots), Q(77_389937) / Q(1_000000));
    }
}
