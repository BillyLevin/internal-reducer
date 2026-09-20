use std::cmp;

#[derive(Debug)]
struct ChoiceSequence {
    choices: Vec<bool>,
    cursor: usize,
}

impl ChoiceSequence {
    fn new(choices: Vec<bool>) -> Self {
        Self { choices, cursor: 0 }
    }

    fn next_choice(&mut self) -> Option<bool> {
        let item = self.choices.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(item)
    }

    fn prepare_replay(&mut self) {
        self.cursor = 0;
    }
}

#[derive(Debug, PartialEq, Eq)]
struct Tree {
    root: Node,
}

impl Tree {
    fn generate(choices: &mut ChoiceSequence) -> Option<Self> {
        Self::create_node(choices).map(|root| Self { root })
    }

    fn create_node(choices: &mut ChoiceSequence) -> Option<Node> {
        if choices.next_choice()? {
            Some(Node::Branch {
                left: Box::new(Self::create_node(choices)?),
                right: Box::new(Self::create_node(choices)?),
            })
        } else {
            Some(Node::Leaf)
        }
    }

    fn has_height_imbalance(&self) -> bool {
        self.root.has_height_imbalance()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Node {
    Leaf,
    Branch { left: Box<Node>, right: Box<Node> },
}

impl Node {
    fn has_height_imbalance(&self) -> bool {
        match self {
            Node::Leaf => false,
            Node::Branch { left, right } => {
                if left.height().abs_diff(right.height()) > 1 {
                    true
                } else {
                    left.has_height_imbalance() || right.has_height_imbalance()
                }
            }
        }
    }

    fn height(&self) -> usize {
        match self {
            Node::Leaf => 1,
            Node::Branch { left, right } => 1 + cmp::max(left.height(), right.height()),
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_generator_generates_leaf() {
        let mut choices = ChoiceSequence::new(vec![false]);

        let expected = Tree { root: Node::Leaf };

        assert_eq!(Tree::generate(&mut choices).unwrap(), expected);
        choices.prepare_replay();
        assert_eq!(Tree::generate(&mut choices).unwrap(), expected);
    }

    #[test]
    fn tree_generator_generates_branch() {
        let mut choices = ChoiceSequence::new(vec![true, false, false]);

        let expected = Tree {
            root: Node::Branch {
                left: Box::new(Node::Leaf),
                right: Box::new(Node::Leaf),
            },
        };

        assert_eq!(Tree::generate(&mut choices).unwrap(), expected);
        choices.prepare_replay();
        assert_eq!(Tree::generate(&mut choices).unwrap(), expected);
    }

    #[test]
    fn tree_generator_fails_for_incomplete_branch() {
        let mut choices = ChoiceSequence::new(vec![true, false]);

        let expected = None;

        assert_eq!(Tree::generate(&mut choices), expected);
        choices.prepare_replay();
        assert_eq!(Tree::generate(&mut choices), expected);
    }

    #[test]
    fn detects_height_imbalance() {
        // LLM-generated test cases
        let cases = [
            ("0", false),      // Single leaf
            ("100", false),    // Two leaves
            ("10100", false),  // Height difference of exactly 1
            ("1010100", true), // Root's child heights differ by 2
            ("1110000", true), // Same imbalance, leaning left
            // Equal-height children at root, but imbalance below it:
            (concat!("1", "1010100", "1010100"), true),
        ];

        for (bits, expected) in cases {
            let mut choices = ChoiceSequence::new(bits.chars().map(|c| c == '1').collect());
            let tree = Tree::generate(&mut choices).unwrap();

            assert_eq!(choices.cursor, bits.len());
            assert_eq!(tree.has_height_imbalance(), expected);
        }
    }
}
