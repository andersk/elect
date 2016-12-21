use gmp::mpq::Mpq;
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::vec::Vec;

struct BallotState {
    visited: bool,
    prev: usize,
    edge_flow: Box<[Mpq]>,
    count: i32,
}

#[derive(Clone)]
struct CandidateState {
    prev: (usize, usize),
    count: i32,
}

pub fn strength<Ballot>(num_seats: usize, ballots: &[(Ballot, Mpq)]) -> Mpq
    where Ballot: Borrow<[usize]>
{
    let candidate_ballots = &mut vec![Vec::new(); num_seats][..];
    for (b, &(ref cs, _)) in ballots.iter().enumerate() {
        for (i, &c) in cs.borrow().iter().enumerate() {
            candidate_ballots[c].push((b, i));
        }
    }
    let ballot_states = &mut ballots.iter()
        .map(|&(ref cs, ref w)| {
            let mut edge_flow = vec![Mpq::zero(); 1 + cs.borrow().len()].into_boxed_slice();
            edge_flow[0] = w.clone();
            BallotState {
                visited: false,
                prev: !0,
                edge_flow: edge_flow,
                count: 0,
            }
        })
        .collect::<Vec<_>>()[..];
    let candidate_states = &mut vec![CandidateState {
        prev: (!0, !0),
        count: 0,
    }; num_seats][..];

    let mut total_flow = Mpq::zero();
    let mut queue = VecDeque::new();

    loop {
        for (b, bs) in ballot_states.iter_mut().enumerate() {
            if !bs.edge_flow[0].is_zero() {
                bs.visited = true;
                bs.prev = 0;
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
                    for (i, &c) in cs.borrow().iter().enumerate() {
                        if candidate_states[c].prev != (!0, !0) {
                            continue;
                        }
                        candidate_states[c].prev = (b, i);
                        found.push(c);
                        if found.len() == num_seats {
                            break 'search;
                        }
                        for &(b1, i1) in &candidate_ballots[c] {
                            if ballot_states[b1].edge_flow[i1 + 1].is_zero() ||
                               ballot_states[b1].visited {
                                continue;
                            }
                            ballot_states[b1].visited = true;
                            ballot_states[b1].prev = i1 + 1;
                            queue.push_back(b1);
                        }
                    }
                }
            }
        }

        let mut sunk = 0;
        for &c in found.iter().rev() {
            let (b, _) = candidate_states[c].prev;
            let count = candidate_states[c].count + 1;
            ballot_states[b].count += count;
            let i1 = ballot_states[b].prev;
            if i1 == 0 {
                sunk += count;
            } else {
                let (ref cs, _) = ballots[b];
                candidate_states[cs.borrow()[i1 - 1]].count += count;
            }
        }
        debug_assert_eq!(sunk, num_seats as i32);

        let flow = found.iter()
            .map(|&c| {
                let (b, _) = candidate_states[c].prev;
                &ballot_states[b].edge_flow[ballot_states[b].prev] /
                Mpq::from(ballot_states[b].count as i64)
            })
            .min()
            .unwrap();
        debug_assert!(!flow.is_zero());
        total_flow = total_flow + &flow;

        for c in found {
            let (b, i) = candidate_states[c].prev;
            {
                let edge_flow = &mut ballot_states[b].edge_flow[i + 1];
                *edge_flow = &*edge_flow + &flow * Mpq::from(candidate_states[c].count as i64 + 1);
            }
            if ballot_states[b].count != 0 {
                let edge_flow = &mut ballot_states[b].edge_flow[ballot_states[b].prev];
                *edge_flow = &*edge_flow - &flow * Mpq::from(ballot_states[b].count as i64);
                ballot_states[b].count = 0;
            }

            candidate_states[c].prev = (!0, !0);
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
        let ballots: &[(&[usize], Mpq)] =
            &[(&[0], Q(12)), (&[1], Q(0)), (&[0, 1], Q(51)), (&[], Q(27))];
        assert_eq!(strength(2, ballots), Q(63) / Q(2));
    }

    #[test]
    fn test_strength_2() {
        // Wikipedia
        let ballots: &[(&[usize], Mpq)] = &[(&[0], Q(38)), (&[1], Q(27)), (&[0, 1], Q(12))];
        assert_eq!(strength(2, ballots), Q(77) / Q(2));
    }

    #[test]
    fn test_strength_3() {
        // Schulze’s calcul02.pdf
        let ballots: &[(&[usize], Mpq)] = &[(&[0, 1, 2, 3], Q(36_597383) / Q(1_000000)),
                                            (&[0, 1, 2], Q(5_481150) / Q(1_000000)),
                                            (&[0, 1, 3], Q(13_279131) / Q(1_000000)),
                                            (&[0, 1], Q(4_859413) / Q(1_000000)),
                                            (&[0, 2, 3], Q(35_425375) / Q(1_000000)),
                                            (&[0, 2], Q(5_490934) / Q(1_000000)),
                                            (&[0, 3], Q(22_855333) / Q(1_000000)),
                                            (&[0], Q(19_835570) / Q(1_000000)),
                                            (&[1, 2, 3], Q(22_928716) / Q(1_000000)),
                                            (&[1, 2], Q(5_538309) / Q(1_000000)),
                                            (&[1, 3], Q(13_130227) / Q(1_000000)),
                                            (&[1], Q(6_056291) / Q(1_000000)),
                                            (&[2, 3], Q(23_992772) / Q(1_000000)),
                                            (&[2], Q(16_699207) / Q(1_000000)),
                                            (&[3], Q(98_165759) / Q(1_000000)),
                                            (&[], Q(129_664430) / Q(1_000000))];
        assert_eq!(strength(4, ballots), Q(77_389937) / Q(1_000000));
    }
}
