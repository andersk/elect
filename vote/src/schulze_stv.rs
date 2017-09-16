use std::borrow::Borrow;
use std::cmp::Ordering;
use std::iter::once;
use std::vec::Vec;
use util::combine_dups;

use combination::{decode_combination, encode_combination, make_binomial};
use proportional_completion::proportional_completion;
use schulze::schulze_graph;
use traits::{Weight, WeightOps};
use vote_management::strength;

fn preferred<Group>(
    num_seats: usize,
    seti: &[usize],
    opponent: usize,
    ballot: &[Group],
) -> Box<[Ordering]>
where
    Group: Borrow<[usize]>,
{
    let mut v = vec![Ordering::Less; num_seats].into_boxed_slice();
    for group in ballot {
        if group.borrow().iter().any(|&i| i == opponent) {
            for &c in group.borrow() {
                if seti[c] != !0 {
                    v[seti[c]] = Ordering::Equal;
                }
            }
            return v;
        }
        for &c in group.borrow() {
            if seti[c] != !0 {
                v[seti[c]] = Ordering::Greater;
            }
        }
    }
    for o in &mut *v {
        if *o == Ordering::Less {
            *o = Ordering::Equal;
        }
    }
    v
}

fn replacements(set: &[usize], opponent: usize) -> Box<[Box<[usize]>]> {
    let k = set.binary_search(&opponent).unwrap_err();
    (0..k)
        .map(|i| {
            set[0..i]
                .iter()
                .cloned()
                .chain(set[i + 1..k].iter().cloned())
                .chain(once(opponent))
                .chain(set[k..].iter().cloned())
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .chain((k..set.len()).map(|i| {
            set[0..k]
                .iter()
                .cloned()
                .chain(once(opponent))
                .chain(set[k..i].iter().cloned())
                .chain(set[i + 1..].iter().cloned())
                .collect::<Vec<_>>()
                .into_boxed_slice()
        }))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

fn all_strengths<W, Group, Groups>(
    num_candidates: usize,
    num_seats: usize,
    ballots: &[(Groups, W)],
) -> Box<[Box<[W]>]>
where
    W: Weight,
    for<'w> &'w W: WeightOps<W>,
    Group: Borrow<[usize]>,
    Groups: Borrow<[Group]>,
{
    let binomial = make_binomial(num_candidates, num_seats);
    let num_combinations = binomial[num_candidates][num_seats];

    (0..num_combinations)
        .map(|m| {
            let set = decode_combination(&binomial, num_seats, m);
            let seti = &mut vec![!0; num_candidates][..];
            for (i, &c) in set.iter().enumerate() {
                seti[c] = i;
            }
            (0..num_candidates)
                .map(|opponent| if seti[opponent] != !0 {
                    W::zero()
                } else {
                    let patterns = ballots
                        .iter()
                        .map(|&(ref groups, ref w)| {
                            (preferred(num_seats, seti, opponent, groups.borrow()), w)
                        })
                        .collect::<Vec<_>>();
                    let completed =
                        proportional_completion(patterns.iter().map(|&(ref a, w)| (&a[..], w)));
                    strength(
                        num_seats,
                        &completed
                            .iter()
                            .map(|&(ref a, ref w)| (&**a, (*w).clone()))
                            .collect::<Vec<_>>()[..],
                    )
                })
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

pub fn schulze_stv<W, Group, Groups>(
    num_candidates: usize,
    num_seats: usize,
    ballots: &[(Groups, W)],
) -> Box<[Box<[usize]>]>
where
    W: Weight,
    for<'w> &'w W: WeightOps<W>,
    Group: Borrow<[usize]>,
    Groups: Borrow<[Group]>,
{
    let binomial = &make_binomial(num_candidates, num_seats);
    let strengths = all_strengths(num_candidates, num_seats, ballots);

    let mut defeats = strengths
        .iter()
        .enumerate()
        .flat_map(move |(m, strength)| {
            let set = decode_combination(binomial, num_seats, m);
            let mut setv = vec![false; num_candidates];
            for &i in &*set {
                setv[i] = true;
            }
            (0..num_candidates)
                .filter(move |&opponent| !setv[opponent])
                .flat_map(move |opponent| {
                    let r = replacements(&set, opponent);
                    r.into_iter()
                        .map(move |set1| {
                            let m1 = encode_combination(binomial, set1);
                            (&strength[opponent], (m, m1))
                        })
                        .collect::<Vec<_>>()
                })
        })
        .collect::<Vec<_>>();

    defeats.sort_by(|a, b| b.0.cmp(a.0));
    let defeat_groups = &combine_dups(
        defeats,
        |a, b| a.0.fuzzy_eq(b.0),
        |a| vec![a],
        |mut a, b| {
            a.push(b);
            a
        },
    ).iter()
        .map(|a| {
            a.iter()
                .map(|&(_, g)| g)
                .collect::<Vec<_>>()
                .into_boxed_slice()
        })
        .collect::<Vec<_>>();

    schulze_graph(strengths.len(), defeat_groups)
        .iter()
        .map(move |&c| decode_combination(binomial, num_seats, c))
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use gmp::mpq::Mpq;

    use combination::{encode_combination, make_binomial};
    use super::{all_strengths, replacements, schulze_stv};

    const Q: fn(i64) -> Mpq = Mpq::from;

    #[test]
    fn test_replacements() {
        let set = vec![2, 4, 6, 8, 10, 12];
        let opponent = 9;
        let expected: &[Box<[usize]>] = &[
            Box::new([4, 6, 8, 9, 10, 12]),
            Box::new([2, 6, 8, 9, 10, 12]),
            Box::new([2, 4, 8, 9, 10, 12]),
            Box::new([2, 4, 6, 9, 10, 12]),
            Box::new([2, 4, 6, 8, 9, 12]),
            Box::new([2, 4, 6, 8, 9, 10]),
        ];
        assert_eq!(*replacements(&set, opponent), *expected);
    }

    #[test]
    fn test_schulze_stv_1() {
        // Schulze’s schulze2.pdf
        let (a, b, c, d, e) = (0, 1, 2, 3, 4);
        let ballots: &[(&[&[usize]], Mpq)] = &[
            (&[&[a], &[b], &[c], &[d], &[e]], Q(60)),
            (&[&[a], &[c], &[e], &[b], &[d]], Q(45)),
            (&[&[a], &[d], &[b], &[e], &[c]], Q(30)),
            (&[&[a], &[e], &[d], &[c], &[b]], Q(15)),
            (&[&[b], &[a], &[e], &[d], &[c]], Q(12)),
            (&[&[b], &[c], &[d], &[e], &[a]], Q(48)),
            (&[&[b], &[d], &[a], &[c], &[e]], Q(39)),
            (&[&[b], &[e], &[c], &[a], &[d]], Q(21)),
            (&[&[c], &[a], &[d], &[b], &[e]], Q(27)),
            (&[&[c], &[b], &[a], &[e], &[d]], Q(9)),
            (&[&[c], &[d], &[e], &[a], &[b]], Q(51)),
            (&[&[c], &[e], &[b], &[d], &[a]], Q(33)),
            (&[&[d], &[a], &[c], &[e], &[b]], Q(42)),
            (&[&[d], &[b], &[e], &[c], &[a]], Q(18)),
            (&[&[d], &[c], &[b], &[a], &[e]], Q(6)),
            (&[&[d], &[e], &[a], &[b], &[c]], Q(54)),
            (&[&[e], &[a], &[b], &[c], &[d]], Q(57)),
            (&[&[e], &[b], &[d], &[a], &[c]], Q(36)),
            (&[&[e], &[c], &[a], &[d], &[b]], Q(24)),
            (&[&[e], &[d], &[c], &[b], &[a]], Q(3)),
        ];
        let binomial = make_binomial(5, 3);
        let strengths = &mut vec![Vec::new().into_boxed_slice(); 10][..];
        let expected: &[(&[usize], Box<[Mpq]>)] = &[
            (&[a, b, c], Box::new([Q(0), Q(0), Q(0), Q(169), Q(152)])),
            (&[a, b, d], Box::new([Q(0), Q(0), Q(162), Q(0), Q(159)])),
            (&[a, b, e], Box::new([Q(0), Q(0), Q(168), Q(153), Q(0)])),
            (&[a, c, d], Box::new([Q(0), Q(158), Q(0), Q(0), Q(163)])),
            (&[a, c, e], Box::new([Q(0), Q(164), Q(0), Q(157), Q(0)])),
            (&[a, d, e], Box::new([Q(0), Q(167), Q(154), Q(0), Q(0)])),
            (&[b, c, d], Box::new([Q(141), Q(0), Q(0), Q(0), Q(165)])),
            (&[b, c, e], Box::new([Q(146), Q(0), Q(0), Q(160), Q(0)])),
            (&[b, d, e], Box::new([Q(151), Q(0), Q(155), Q(0), Q(0)])),
            (&[c, d, e], Box::new([Q(156), Q(150), Q(0), Q(0), Q(0)])),
        ];
        for &(ref m, ref v) in expected {
            strengths[encode_combination(&binomial, m)] = v.clone();
        }
        assert_eq!(*all_strengths(5, 3, ballots), *strengths);
        let expected: &[Box<[usize]>] = &[Box::new([a, d, e])];
        assert_eq!(*schulze_stv(5, 3, ballots), *expected);
    }

    #[test]
    fn test_schulze_stv_2() {
        // Wikipedia
        let ballots: &[(&[&[usize]], Mpq)] = &[
            (&[&[0], &[1], &[2]], Q(12)),
            (&[&[0], &[2], &[1]], Q(26)),
            (&[&[0], &[2], &[1]], Q(12)),
            (&[&[2], &[0], &[1]], Q(13)),
            (&[&[1]], Q(27)),
        ];
        let binomial = make_binomial(3, 2);
        let strengths = &mut vec![Vec::new().into_boxed_slice(); 3][..];
        let expected: &[(&[usize], Box<[Mpq]>)] = &[
            (&[0, 1], Box::new([Q(0), Q(0), Q(77) / Q(2)])),
            (&[0, 2], Box::new([Q(0), Q(63) / Q(2), Q(0)])),
            (&[1, 2], Box::new([Q(130) / Q(7), Q(0), Q(0)])),
        ];
        for &(ref m, ref v) in expected {
            strengths[encode_combination(&binomial, m)] = v.clone();
        }
        assert_eq!(*all_strengths(3, 2, ballots), *strengths);
        let expected: &[Box<[usize]>] = &[Box::new([0, 1])];
        assert_eq!(*schulze_stv(3, 2, ballots), *expected);
    }
}
