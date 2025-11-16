use std::cell::RefCell;
use std::rc::Rc;
use std::usize;

pub type NodeRef = Option<Rc<RefCell<RopeNode>>>;

#[derive(Debug, Clone)]
pub struct RopeNode {
    weight: usize,
    data: Option<String>,
    left: NodeRef,
    right: NodeRef,
}

impl RopeNode {
    pub fn new_leaf(data: &str) -> NodeRef {
        Some(Rc::new(RefCell::new(RopeNode {
            weight: data.len(),
            data: Some(data.to_string()),
            left: None,
            right: None,
        })))
    }

    pub fn new_internal(left: NodeRef, right: NodeRef) -> NodeRef {
        let weight = match &left {
            Some(l) => l.borrow().length(),
            None => 0,
        };
        Some(Rc::new(RefCell::new(RopeNode {
            weight,
            data: None,
            left,
            right,
        })))
    }

    pub fn length(&self) -> usize {
        if self.left.is_none() && self.right.is_none() {
            return self.data.as_ref().unwrap().len();
        }
        let left_len = self.left.as_ref().map(|l| l.borrow().length()).unwrap_or(0);
        let right_len = self.right.as_ref().map(|l| l.borrow().length()).unwrap_or(0);
        left_len + right_len
    }
}

pub fn concatenate(left: NodeRef, right: NodeRef) -> NodeRef {
    RopeNode::new_internal(left, right)
}

pub fn index_at(node: &NodeRef, mut i: usize) -> Option<char> {
    if let Some(n) = node {
        let node_borrow = n.borrow();
        if node_borrow.left.is_none() && node_borrow.right.is_none() {
            return node_borrow.data.as_ref()?.chars().nth(i);
        }

        if i < node_borrow.weight {
            return index_at(&node_borrow.left, i);
        } else {
            i -= node_borrow.weight;
            return index_at(&node_borrow.right, i);
        }
    }
    None
}

pub fn split(node: &NodeRef, i: usize) -> (NodeRef, NodeRef) {
    if node.is_none() {
        return (None, None);
    }

    let n = node.as_ref().unwrap();
    let nb = n.borrow();

    if nb.left.is_none() && nb.right.is_none() {
        let data = nb.data.as_ref().unwrap();
        let left_str = data.chars().take(i).collect::<String>();
        let right_str = data.chars().skip(i).collect::<String>();
        return (
            RopeNode::new_leaf(&left_str),
            RopeNode::new_leaf(&right_str),
        );
    }

    if i < nb.weight {
        let (left_split, right_split) = split(&nb.left, i);
        let new_right = concatenate(right_split, nb.right.clone());
        return (left_split, new_right);
    } else {
        let (left_split, right_split) = split(&nb.right, i - nb.weight);
        let new_left = concatenate(nb.left.clone(), left_split);
        return (new_left, right_split);
    }
}

pub fn insert(root: NodeRef, i: usize, text: &str) -> NodeRef {
    let (left, right) = split(&root, i);
    let new_node = RopeNode::new_leaf(text);
    concatenate(concatenate(left, new_node), right)
}

pub fn delete(root: NodeRef, start: usize, end: usize) -> NodeRef {
    let (left, right1) = split(&root, start);
    let (_, right) = split(&right1, end - start);
    concatenate(left, right)
}

pub fn report(node: &NodeRef) -> String {
    if let Some(n) = node {
        let nb = n.borrow();
        if nb.right.is_none() && nb.left.is_none() {
            return nb.data.clone().unwrap_or_default();
        }
        return format!("{}{}", report(&nb.left), report(&nb.right));
    }
    String::new()
}
