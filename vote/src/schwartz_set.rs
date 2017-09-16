use std::borrow::Borrow;
use std::cmp::min;
use std::vec::Vec;

#[derive(Clone)]
enum Node {
    Unvisited,
    OnStack(usize),
    Done,
}

struct State<'a> {
    nodes: &'a mut [Node],
    stack: Vec<usize>,
    schwartz: Vec<usize>,
}

fn search<Defeaters>(defeaters: &[Defeaters], state: &mut State, c: usize) -> Option<usize>
where
    Defeaters: Borrow<[usize]>,
{
    let mut lowlink = state.stack.len();
    state.nodes[c] = Node::OnStack(lowlink);
    state.stack.push(c);

    for &c1 in defeaters[c].borrow() {
        match state.nodes[c1] {
            Node::Unvisited => if let Some(lowlink1) = search(defeaters, state, c1) {
                lowlink = min(lowlink, lowlink1);
            },
            Node::OnStack(index1) => {
                lowlink = min(lowlink, index1);
            }
            Node::Done => {
                for &c1 in &state.stack {
                    state.nodes[c1] = Node::Done;
                }
                state.stack.clear();
            }
        }
    }

    match state.nodes[c] {
        Node::OnStack(index) if lowlink == index => {
            state.schwartz.extend_from_slice(&state.stack[index..]);
            for &c1 in &state.stack {
                state.nodes[c1] = Node::Done;
            }
            state.stack.clear();
            None
        }
        Node::Done => None,
        _ => Some(lowlink),
    }
}

pub fn schwartz_set<Defeaters>(candidates: &[usize], defeaters: &[Defeaters]) -> Box<[usize]>
where
    Defeaters: Borrow<[usize]>,
{
    let mut state = State {
        nodes: &mut vec![Node::Unvisited; defeaters.len()][..],
        stack: Vec::new(),
        schwartz: Vec::new(),
    };

    for &c in candidates {
        if let Node::Unvisited = state.nodes[c] {
            search(defeaters, &mut state, c);
        }
    }

    state.schwartz.into_boxed_slice()
}

#[cfg(test)]
mod tests {
    use super::schwartz_set;

    fn sorted<T: Clone + Ord>(v: &[T]) -> Box<[T]> {
        let mut v1 = v.to_vec().into_boxed_slice();
        v1.sort();
        v1
    }

    #[test]
    fn test_schwartz_set_1() {
        let defeaters: &[&[usize]] = &[&[], &[0]];
        assert_eq!(*sorted(&schwartz_set(&[0, 1], defeaters)), [0]);
    }

    #[test]
    fn test_schwartz_set_2() {
        let defeaters: &[&[usize]] = &[&[1], &[]];
        assert_eq!(*sorted(&schwartz_set(&[0, 1], defeaters)), [1]);
    }

    #[test]
    fn test_schwartz_set_3() {
        let defeaters: &[&[usize]] = &[&[], &[]];
        assert_eq!(*sorted(&schwartz_set(&[0, 1], defeaters)), [0, 1]);
    }

    #[test]
    fn test_schwartz_set_4() {
        let defeaters: &[&[usize]] = &[&[2], &[0], &[1]];
        assert_eq!(*sorted(&schwartz_set(&[0, 1, 2], defeaters)), [0, 1, 2]);
    }

    #[test]
    fn test_schwartz_set_5() {
        let defeaters: &[&[usize]] = &[&[2, 3], &[0], &[1], &[]];
        assert_eq!(*sorted(&schwartz_set(&[0, 1, 2, 3], defeaters)), [3]);
    }
}
