use gmp::mpq::Mpq;
use std::collections::HashMap;
use std::cmp::Ordering;
use std::vec::Vec;

fn is_equal(&o: &Ordering) -> bool {
    o == Ordering::Equal
}

fn count_equal(a: &[Ordering]) -> usize {
    a.iter().cloned().filter(is_equal).count()
}

pub fn proportional_completion<'a, Orderings, Patterns>(patterns: Patterns)
                                                        -> Box<[(Box<[bool]>, Mpq)]>
    where Orderings: Iterator<Item = Ordering>,
          Patterns: Iterator<Item = (Orderings, &'a Mpq)>
{
    let mut patterns_by_count = vec![HashMap::<Box<[Ordering]>, Mpq>::new()];
    for (a, w) in patterns {
        let a = a.collect::<Vec<_>>().into_boxed_slice();
        let t = count_equal(&a);
        if t >= patterns_by_count.len() {
            patterns_by_count.resize(t + 1, HashMap::new());
        }
        let e = patterns_by_count[t].entry(a).or_insert_with(Mpq::zero);
        *e = &*e + w;
    }
    while patterns_by_count.len() > 1 {
        let mut tied = patterns_by_count.pop().unwrap();
        let mut tied = tied.drain().collect::<Vec<_>>();
        while let Some((a, w)) = tied.pop() {
            let mut replacements = HashMap::new();
            let mut total = Mpq::zero();
            for (&ref a1, &ref w1) in
                patterns_by_count.iter()
                    .flat_map(|&ref it| it)
                    .chain(tied.iter().map(|&(ref a1, ref w1)| (a1, w1))) {
                if a.iter()
                    .zip(a1.iter())
                    .any(|(&o, &o1)| o == Ordering::Equal && o1 != Ordering::Equal) {
                    let a2 = a.iter()
                        .zip(a1.iter())
                        .map(|(&o, &o1)| if o == Ordering::Equal { o1 } else { o })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let e = replacements.entry(a2).or_insert_with(Mpq::zero);
                    *e = &*e + w1;
                    total = total + w1;
                }
            }
            if total.is_zero() {
                let scale = w / Mpq::from(2u64);
                for &o1 in &[Ordering::Greater, Ordering::Less] {
                    let a1 = a.iter()
                        .map(|&o| if o == Ordering::Equal { o1 } else { o })
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let e = patterns_by_count[count_equal(&a1)].entry(a1).or_insert_with(Mpq::zero);
                    *e = &*e + &scale;
                }
            } else {
                let scale = w / total;
                for (&ref a1, &ref w1) in &replacements {
                    let e = patterns_by_count[count_equal(a1)]
                        .entry(a1.clone())
                        .or_insert_with(Mpq::zero);
                    *e = &*e + w1 * &scale;
                }
            }
        }
    }
    patterns_by_count.pop()
        .unwrap()
        .iter()
        .map(|(&ref a, &ref w)| {
            (a.iter().map(|&o| o == Ordering::Greater).collect::<Vec<_>>().into_boxed_slice(),
             w.clone())
        })
        .collect::<Vec<_>>()
        .into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use gmp::mpq::Mpq;
    use gmp::mpz::Mpz;
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
        let patterns: &[(&[i32], Mpq)] = &[(&[1, 1, 1, 1], Q(46)),
                                           (&[1, 1, 1, 2], Q(15)),
                                           (&[1, 1, 1, 3], Q(24)),
                                           (&[1, 1, 2, 1], Q(8)),
                                           (&[1, 1, 2, 2], Q(10)),
                                           (&[1, 1, 3, 1], Q(10)),
                                           (&[1, 1, 3, 3], Q(19)),
                                           (&[1, 2, 1, 1], Q(8)),
                                           (&[1, 2, 1, 2], Q(29)),
                                           (&[1, 2, 2, 1], Q(10)),
                                           (&[1, 2, 2, 2], Q(26)),
                                           (&[1, 3, 1, 1], Q(10)),
                                           (&[1, 3, 1, 3], Q(15)),
                                           (&[1, 3, 2, 1], Q(1)),
                                           (&[1, 3, 3, 1], Q(9)),
                                           (&[1, 3, 3, 3], Q(41)),
                                           (&[2, 1, 1, 1], Q(3)),
                                           (&[2, 1, 1, 2], Q(5)),
                                           (&[2, 1, 2, 1], Q(5)),
                                           (&[2, 1, 2, 2], Q(10)),
                                           (&[2, 2, 1, 1], Q(7)),
                                           (&[2, 2, 1, 2], Q(22)),
                                           (&[2, 2, 2, 1], Q(14)),
                                           (&[2, 2, 2, 2], Q(23)),
                                           (&[2, 3, 3, 3], Q(1)),
                                           (&[3, 1, 1, 1], Q(1)),
                                           (&[3, 1, 1, 3], Q(3)),
                                           (&[3, 1, 3, 1], Q(4)),
                                           (&[3, 1, 3, 3], Q(5)),
                                           (&[3, 3, 1, 1], Q(4)),
                                           (&[3, 3, 1, 3], Q(11)),
                                           (&[3, 3, 3, 1], Q(6)),
                                           (&[3, 3, 3, 3], Q(55))];
        let patterns_iter = patterns.iter().map(|&(ref a, ref w)| (a.iter().map(|n| 2.cmp(n)), w));
        let expected: &[(&[i32], Mpq)] =
            &[(&[1, 1, 1, 1],
               Mpq::ratio(&Mpz::from_str("45513455366183031714312799").unwrap(),
                          &Mpz::from_str("392675963004164103979050").unwrap())),
              (&[1, 1, 1, 3], Q(313141398725389) / Q(5802708856700)),
              (&[1, 1, 3, 1], Q(398610303226324835) / Q(17955446458922532)),
              (&[1, 1, 3, 3], Q(56713304320) / Q(2097364647)),
              (&[1, 3, 1, 1],
               Mpq::ratio(&Mpz::from_str("323263789854293839067").unwrap(),
                          &Mpz::from_str("10819009863732307590").unwrap())),
              (&[1, 3, 1, 3], Q(2664088261) / Q(79938130)),
              (&[1, 3, 3, 1], Q(10276259658824) / Q(618385674987)),
              (&[1, 3, 3, 3], Q(14567002) / Q(288933)),
              (&[3, 1, 1, 1], Q(9026619205849313) / Q(2353019619161400)),
              (&[3, 1, 1, 3], Q(25937418993) / Q(4983004600)),
              (&[3, 1, 3, 1], Q(16829300885) / Q(2878135428)),
              (&[3, 1, 3, 3], Q(2025680) / Q(300181)),
              (&[3, 3, 1, 1], Q(57854720993) / Q(8103800865)),
              (&[3, 3, 1, 3], Q(4456496) / Q(291745)),
              (&[3, 3, 3, 1], Q(837524) / Q(99123)),
              (&[3, 3, 3, 3], Q(40878) / Q(703))];
        let expected = expected.iter()
            .map(|&(ref a, ref w)| {
                (a.iter()
                     .map(|&n| n == 1)
                     .collect::<Vec<_>>()
                     .into_boxed_slice(),
                 w.clone())
            })
            .collect::<Vec<_>>();
        assert_eq!(*sorted(&proportional_completion(patterns_iter)),
                   *sorted(&expected));
    }
}
