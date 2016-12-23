use gmp::mpq::Mpq;
use std::cmp::Ordering;
use std::vec::Vec;
use util::{combine_dups2, merge_combine};

fn encode_pattern(a: &[Ordering]) -> (usize, usize) {
    a.iter().rev().fold((0, 0), |(eq, gt), &o| {
        (eq << 1 | if o == Ordering::Equal { 1 } else { 0 },
         gt << 1 | if o == Ordering::Greater { 1 } else { 0 })
    })
}

fn decode_bits(mut gt: usize) -> Box<[usize]> {
    let mut cs = Vec::new();
    cs.reserve(gt.count_ones() as usize);
    while gt != 0 {
        let k = gt.trailing_zeros();
        cs.push(k as usize);
        gt &= !(1 << k);
    }
    cs.into_boxed_slice()
}

pub fn proportional_completion<'a, Patterns>(patterns_iter: Patterns) -> Box<[(Box<[usize]>, Mpq)]>
    where Patterns: Iterator<Item = (&'a [Ordering], &'a Mpq)>
{
    let mut patterns = patterns_iter.filter(|&(_, w)| !w.is_zero())
        .map(|(a, w)| (encode_pattern(a), w))
        .collect::<Vec<_>>();
    patterns.sort_by(|a, b| a.0.cmp(&b.0));
    let mut patterns = combine_dups2(patterns,
                                     |a, b| a.0 == b.0,
                                     |a| (a.0, a.1.clone()),
                                     |a, b| (a.0, a.1 + b.1),
                                     |a, b| (a.0, a.1 + b.1));
    let total = patterns.iter().fold(Mpq::zero(), |acc, &(_, ref w)| acc + w);

    while let Some(&((eq, _), _)) = patterns.last() {
        if eq == 0 {
            return patterns.into_iter()
                .map(|((_, gt), w)| (decode_bits(gt), w))
                .collect::<Vec<_>>()
                .into_boxed_slice();
        }

        let i = if let Some(i) = patterns.iter().rposition(|&((eq1, _), _)| eq1 != eq) {
            i + 1
        } else {
            return patterns.iter()
                .map(|&((_, gt), ref w)| (decode_bits(gt), w / Mpq::from(2u64)))
                .chain(patterns.iter()
                    .map(|&((_, gt), ref w)| (decode_bits(gt | eq), w / Mpq::from(2u64))))
                .collect::<Vec<_>>()
                .into_boxed_slice();
        };

        let scale = patterns[i..].iter().fold(total.clone(), |acc, &(_, ref w)| acc - w).invert();

        let breakers = {
            let mut breakers = patterns[..i]
                .iter()
                .map(|&((eq1, gt1), ref w1)| {
                    debug_assert!(eq1 < eq);
                    ((eq & eq1, eq & gt1), w1)
                })
                .collect::<Vec<_>>();
            breakers.sort_by(|a, b| a.0.cmp(&b.0));
            combine_dups2(breakers,
                          |a, b| a.0 == b.0,
                          |a| (a.0, a.1.clone()),
                          |a, b| (a.0, a.1 + b.1),
                          |a, b| (a.0, a.1 + b.1))
        };

        let mut new_patterns = patterns.drain(i..)
            .flat_map(|((eq_, gt), w)| {
                debug_assert_eq!(eq_, eq);
                let w_scaled = w * &scale;
                breakers.iter().map(move |&((eq1, gt1), ref w1)| {
                    debug_assert!(eq1 < eq);
                    ((eq1, gt | gt1), w1 * &w_scaled)
                })
            })
            .collect::<Vec<_>>();
        new_patterns.sort_by(|a, b| a.0.cmp(&b.0));

        patterns = merge_combine(patterns,
                                 new_patterns,
                                 |a, b| a.0.cmp(&b.0),
                                 |a, b| (a.0, a.1 + b.1));
        debug_assert_eq!(patterns.iter().fold(Mpq::zero(), |acc, &(_, ref w)| acc + w),
                         total);
    }
    Box::new([])
}

#[cfg(test)]
mod tests {
    use gmp::mpq::Mpq;
    use gmp::mpz::Mpz;
    use std::cmp::Ordering::{self, Greater as _1, Equal as _2, Less as _3};
    use std::str::FromStr;

    use super::proportional_completion;

    const Q: fn(i64) -> Mpq = Mpq::from;

    fn sorted<T: Clone + Ord>(v: &[T]) -> Box<[T]> {
        let mut v1 = v.to_vec().into_boxed_slice();
        v1.sort();
        v1
    }

    #[test]
    fn test_proportional_completion_1() {
        // Schulze’s calcul02.pdf
        let patterns: &[(&[Ordering], &Mpq)] = &[(&[_1, _1, _1, _1], &Q(46)),
                                                 (&[_1, _1, _1, _2], &Q(15)),
                                                 (&[_1, _1, _1, _3], &Q(24)),
                                                 (&[_1, _1, _2, _1], &Q(8)),
                                                 (&[_1, _1, _2, _2], &Q(10)),
                                                 (&[_1, _1, _3, _1], &Q(10)),
                                                 (&[_1, _1, _3, _3], &Q(19)),
                                                 (&[_1, _2, _1, _1], &Q(8)),
                                                 (&[_1, _2, _1, _2], &Q(29)),
                                                 (&[_1, _2, _2, _1], &Q(10)),
                                                 (&[_1, _2, _2, _2], &Q(26)),
                                                 (&[_1, _3, _1, _1], &Q(10)),
                                                 (&[_1, _3, _1, _3], &Q(15)),
                                                 (&[_1, _3, _2, _1], &Q(1)),
                                                 (&[_1, _3, _3, _1], &Q(9)),
                                                 (&[_1, _3, _3, _3], &Q(41)),
                                                 (&[_2, _1, _1, _1], &Q(3)),
                                                 (&[_2, _1, _1, _2], &Q(5)),
                                                 (&[_2, _1, _2, _1], &Q(5)),
                                                 (&[_2, _1, _2, _2], &Q(10)),
                                                 (&[_2, _2, _1, _1], &Q(7)),
                                                 (&[_2, _2, _1, _2], &Q(22)),
                                                 (&[_2, _2, _2, _1], &Q(14)),
                                                 (&[_2, _2, _2, _2], &Q(23)),
                                                 (&[_2, _3, _3, _3], &Q(1)),
                                                 (&[_3, _1, _1, _1], &Q(1)),
                                                 (&[_3, _1, _1, _3], &Q(3)),
                                                 (&[_3, _1, _3, _1], &Q(4)),
                                                 (&[_3, _1, _3, _3], &Q(5)),
                                                 (&[_3, _3, _1, _1], &Q(4)),
                                                 (&[_3, _3, _1, _3], &Q(11)),
                                                 (&[_3, _3, _3, _1], &Q(6)),
                                                 (&[_3, _3, _3, _3], &Q(55))];
        let expected: &[(&[Ordering], Mpq)] =
            &[(&[_1, _1, _1, _1],
               Mpq::ratio(&Mpz::from_str("45513455366183031714312799").unwrap(),
                          &Mpz::from_str("392675963004164103979050").unwrap())),
              (&[_1, _1, _1, _3], Q(313141398725389) / Q(5802708856700)),
              (&[_1, _1, _3, _1], Q(398610303226324835) / Q(17955446458922532)),
              (&[_1, _1, _3, _3], Q(56713304320) / Q(2097364647)),
              (&[_1, _3, _1, _1],
               Mpq::ratio(&Mpz::from_str("323263789854293839067").unwrap(),
                          &Mpz::from_str("10819009863732307590").unwrap())),
              (&[_1, _3, _1, _3], Q(2664088261) / Q(79938130)),
              (&[_1, _3, _3, _1], Q(10276259658824) / Q(618385674987)),
              (&[_1, _3, _3, _3], Q(14567002) / Q(288933)),
              (&[_3, _1, _1, _1], Q(9026619205849313) / Q(2353019619161400)),
              (&[_3, _1, _1, _3], Q(25937418993) / Q(4983004600)),
              (&[_3, _1, _3, _1], Q(16829300885) / Q(2878135428)),
              (&[_3, _1, _3, _3], Q(2025680) / Q(300181)),
              (&[_3, _3, _1, _1], Q(57854720993) / Q(8103800865)),
              (&[_3, _3, _1, _3], Q(4456496) / Q(291745)),
              (&[_3, _3, _3, _1], Q(837524) / Q(99123)),
              (&[_3, _3, _3, _3], Q(40878) / Q(703))];
        let expected = expected.iter()
            .map(|&(ref a, ref w)| {
                (a.iter()
                     .enumerate()
                     .filter(|&(_, &o)| o == _1)
                     .map(|(i, _)| i)
                     .collect::<Vec<_>>()
                     .into_boxed_slice(),
                 w.clone())
            })
            .collect::<Vec<_>>();
        assert_eq!(*sorted(&proportional_completion(patterns.iter().cloned())),
                   *sorted(&expected));
    }

    #[test]
    fn test_proportional_completion_2() {
        let patterns: &[(&[Ordering], &Mpq)] = &[(&[_1, _3], &Q(1)), (&[_3, _2], &Q(1))];
        let expected: &[(Box<[usize]>, Mpq)] = &[(Box::new([]), Q(1)), (Box::new([0]), Q(1))];
        assert_eq!(*sorted(&proportional_completion(patterns.iter().cloned())),
                   *sorted(&expected));
    }
}
