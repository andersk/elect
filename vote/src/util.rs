use std::cmp::Ordering;

pub fn combine_dups<A, B, I, Eq, One, More>(i: I, eq: Eq, one: One, more: More) -> Vec<B>
where
    I: IntoIterator<Item = A>,
    Eq: Fn(&A, &A) -> bool,
    One: Fn(A) -> B,
    More: Fn(B, A) -> B,
{
    combine_dups2(i, eq, |a| one(a), |a, b| more(one(a), b), |a, b| more(a, b))
}

pub fn combine_dups2<A, B, I, Eq, One, Two, More>(
    i: I,
    eq: Eq,
    one: One,
    two: Two,
    more: More,
) -> Vec<B>
where
    I: IntoIterator<Item = A>,
    Eq: Fn(&A, &A) -> bool,
    One: Fn(A) -> B,
    Two: Fn(A, A) -> B,
    More: Fn(B, A) -> B,
{
    let mut i = i.into_iter();
    let mut v = Vec::with_capacity(i.size_hint().0);
    if let Some(mut a) = i.next() {
        'outer: while let Some(b) = i.next() {
            if !eq(&a, &b) {
                v.push(one(a));
                a = b;
                continue;
            }
            if let Some(mut c) = i.next() {
                if !eq(&b, &c) {
                    v.push(two(a, b));
                    a = c;
                    continue;
                }
                let mut g = two(a, b);
                while let Some(d) = i.next() {
                    if !eq(&c, &d) {
                        v.push(more(g, c));
                        a = d;
                        continue 'outer;
                    }
                    g = more(g, c);
                    c = d;
                }
                v.push(more(g, c));
                return v;
            } else {
                v.push(two(a, b));
                return v;
            }
        }
        v.push(one(a));
    }
    v
}

pub fn merge_combine<T, I, J, Cmp, Combine>(i: I, j: J, cmp: Cmp, combine: Combine) -> Vec<T>
where
    I: IntoIterator<Item = T>,
    J: IntoIterator<Item = T>,
    Cmp: Fn(&T, &T) -> Ordering,
    Combine: Fn(T, T) -> T,
{
    let (mut i, mut j) = (i.into_iter(), j.into_iter());
    if let Some(mut b) = j.next() {
        let mut v = Vec::with_capacity(i.size_hint().0 + j.size_hint().0 + 1);
        while let Some(a) = i.next() {
            loop {
                match cmp(&a, &b) {
                    Ordering::Greater => {
                        v.push(b);
                        if let Some(c) = j.next() {
                            b = c;
                        } else {
                            v.push(a);
                            v.extend(i);
                            return v;
                        }
                    }
                    Ordering::Equal => {
                        v.push(combine(a, b));
                        if let Some(c) = j.next() {
                            b = c;
                            break;
                        } else {
                            v.extend(i);
                            return v;
                        }
                    }
                    Ordering::Less => {
                        v.push(a);
                        break;
                    }
                }
            }
        }
        v.push(b);
        v.extend(j);
        v
    } else {
        i.collect()
    }
}
