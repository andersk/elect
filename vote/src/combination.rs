use std::cmp::Ordering;

pub fn make_binomial(max_n: usize, max_k: usize) -> Box<[Box<[usize]>]> {
    let mut binomial = vec![vec![0; max_k + 1].into_boxed_slice(); max_n + 1].into_boxed_slice();
    binomial[0][0] = 1;
    for n in 0..max_n {
        binomial[n + 1][0] = 1;
        for k in 0..max_k {
            binomial[n + 1][k + 1] = binomial[n][k] + binomial[n][k + 1];
        }
    }
    binomial
}

pub fn encode_combination(binomial: &[Box<[usize]>], c: &[usize]) -> usize {
    debug_assert!(c.iter().zip(c.iter().skip(1)).all(|(&a, &b)| a < b));
    c.iter().enumerate().map(|(i, &a)| binomial[a][i + 1]).sum()
}

pub fn decode_combination(binomial: &[Box<[usize]>], k: usize, m: usize) -> Box<[usize]> {
    let mut c = vec![0; k].into_boxed_slice();
    let mut mm = m;
    let mut n = binomial.len();
    for i in (0..k).rev() {
        c[i] = binomial[i + 1..n]
            .binary_search_by(|b| if b[i + 1] > mm {
                Ordering::Greater
            } else {
                Ordering::Less
            })
            .unwrap_err() + i;
        mm -= binomial[c[i]][i + 1];
        n = c[i];
    }
    debug_assert_eq!(mm, 0);
    c
}

#[cfg(test)]
mod tests {
    use super::{make_binomial, encode_combination, decode_combination};

    #[test]
    fn test_binomial() {
        let expected: &[Box<[usize]>] = &[Box::new([1, 0, 0, 0, 0, 0]),
                                          Box::new([1, 1, 0, 0, 0, 0]),
                                          Box::new([1, 2, 1, 0, 0, 0]),
                                          Box::new([1, 3, 3, 1, 0, 0]),
                                          Box::new([1, 4, 6, 4, 1, 0]),
                                          Box::new([1, 5, 10, 10, 5, 1]),
                                          Box::new([1, 6, 15, 20, 15, 6]),
                                          Box::new([1, 7, 21, 35, 35, 21]),
                                          Box::new([1, 8, 28, 56, 70, 56]),
                                          Box::new([1, 9, 36, 84, 126, 126]),
                                          Box::new([1, 10, 45, 120, 210, 252])];
        assert_eq!(*make_binomial(10, 5), *expected);
    }

    #[test]
    fn test_encode_combination() {
        let binomial = make_binomial(5, 3);
        assert_eq!(encode_combination(&binomial, &[0, 1, 2]), 0);
        assert_eq!(encode_combination(&binomial, &[0, 1, 3]), 1);
        assert_eq!(encode_combination(&binomial, &[0, 2, 3]), 2);
        assert_eq!(encode_combination(&binomial, &[1, 2, 3]), 3);
        assert_eq!(encode_combination(&binomial, &[0, 1, 4]), 4);
        assert_eq!(encode_combination(&binomial, &[0, 2, 4]), 5);
        assert_eq!(encode_combination(&binomial, &[1, 2, 4]), 6);
        assert_eq!(encode_combination(&binomial, &[0, 3, 4]), 7);
        assert_eq!(encode_combination(&binomial, &[1, 3, 4]), 8);
        assert_eq!(encode_combination(&binomial, &[2, 3, 4]), 9);
        assert_eq!(encode_combination(&binomial, &[0, 1, 5]), 10);
    }

    #[test]
    fn test_decode_combination() {
        let binomial = make_binomial(5, 3);
        assert_eq!(*decode_combination(&binomial, 3, 0), [0, 1, 2]);
        assert_eq!(*decode_combination(&binomial, 3, 1), [0, 1, 3]);
        assert_eq!(*decode_combination(&binomial, 3, 2), [0, 2, 3]);
        assert_eq!(*decode_combination(&binomial, 3, 3), [1, 2, 3]);
        assert_eq!(*decode_combination(&binomial, 3, 4), [0, 1, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 5), [0, 2, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 6), [1, 2, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 7), [0, 3, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 8), [1, 3, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 9), [2, 3, 4]);
        assert_eq!(*decode_combination(&binomial, 3, 10), [0, 1, 5]);
    }
}
