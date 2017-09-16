use std::borrow::Borrow;
use std::vec::Vec;
use schwartz_set::schwartz_set;

pub fn schulze_graph<DefeatGroup>(
    num_candidates: usize,
    defeat_groups: &[DefeatGroup],
) -> Box<[usize]>
where
    DefeatGroup: Borrow<[(usize, usize)]>,
{
    let defeaters = &mut vec![Vec::new(); num_candidates][..];
    for defeat_group in defeat_groups {
        for &(a, b) in defeat_group.borrow() {
            defeaters[b].push(a);
        }
    }
    let mut candidates = (0..num_candidates).collect::<Vec<_>>().into_boxed_slice();
    for defeat_group in defeat_groups.iter().rev() {
        if candidates.len() <= 1 {
            break;
        }
        let schwartz = schwartz_set(&candidates, defeaters);
        candidates = schwartz;
        for &(a, b) in defeat_group.borrow().iter().rev() {
            let a1 = defeaters[b].pop();
            debug_assert_eq!(a1, Some(a));
        }
    }
    candidates
}

#[cfg(test)]
mod tests {
    use super::schulze_graph;

    #[test]
    fn test_schulze_1() {
        // Wikipedia
        let defeat_groups: &[&[(usize, usize)]] = &[
            &[(1, 3)],
            &[(4, 3)],
            &[(0, 3)],
            &[(2, 1)],
            &[(3, 2)],
            &[(4, 1)],
            &[(0, 2)],
            &[(1, 0)],
            &[(2, 4)],
            &[(4, 0)],
        ];
        assert_eq!(*schulze_graph(5, defeat_groups), [4]);
    }
}
