use std::{cmp, fmt};

use crate::ChoiceSequence;

/// An example data structure and its generator. this has nothing to do with the reducer itself, and
/// is interchangeable with any data structure that can be generated based on a choice sequence
#[derive(Debug, PartialEq, Eq)]
pub struct Tree {
    root: Node,
}

impl Tree {
    pub fn generate(choices: &mut ChoiceSequence) -> Option<Self> {
        Self::create_node(choices).map(|root| Self { root })
    }

    fn create_node(choices: &mut ChoiceSequence) -> Option<Node> {
        choices.draw(|choices| {
            if choices.next_choice()? {
                Some(Node::Branch {
                    left: Box::new(Self::create_node(choices)?),
                    right: Box::new(Self::create_node(choices)?),
                })
            } else {
                Some(Node::Leaf)
            }
        })
    }

    pub fn has_height_imbalance(&self) -> bool {
        self.root.has_height_imbalance()
    }
}

impl fmt::Display for Tree {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut nodes = Vec::new();
        self.root.layout(Vec::new(), &mut nodes);

        let max_depth = nodes.iter().map(|node| node.path.len()).max().unwrap_or(0);
        let mut steps = vec![3; max_depth];
        // Keep the compact, equal-length edges of a chain, but spread apart
        // subtrees whose nodes would otherwise occupy the same column.
        loop {
            let collision = nodes.iter().enumerate().find_map(|(i, a)| {
                nodes[..i].iter().find_map(|b| {
                    (a.path.len() == b.path.len() && a.x(&steps) == b.x(&steps)).then(|| {
                        a.path
                            .iter()
                            .zip(&b.path)
                            .position(|(a, b)| a != b)
                            .unwrap()
                    })
                })
            });
            match collision {
                Some(depth) => steps[depth] += 3,
                None => break,
            }
        }

        let min_x = nodes.iter().map(|node| node.x(&steps)).min().unwrap();
        let max_x = nodes.iter().map(|node| node.x(&steps)).max().unwrap();
        let mut rows = vec![0];
        for step in &steps {
            rows.push(rows.last().unwrap() + step);
        }
        let mut canvas = vec![vec![' '; (max_x - min_x + 1) as usize]; rows[max_depth] + 1];

        for node in &nodes {
            let x = node.x(&steps);
            let depth = node.path.len();
            for &child in &node.children {
                let direction = if nodes[child].x(&steps) < x { -1 } else { 1 };
                for step in 1..steps[depth] {
                    canvas[rows[depth] + step][(x - min_x + direction * step as isize) as usize] =
                        if direction < 0 { '/' } else { '\\' };
                }
            }
            canvas[rows[depth]][(x - min_x) as usize] = '○';
        }

        for (index, row) in canvas.iter().enumerate() {
            if index != 0 {
                writeln!(f)?;
            }
            write!(f, "{}", row.iter().collect::<String>().trim_end())?;
        }
        Ok(())
    }
}

struct PlacedNode {
    path: Vec<bool>,
    children: Vec<usize>,
}

impl PlacedNode {
    fn x(&self, steps: &[usize]) -> isize {
        self.path
            .iter()
            .zip(steps)
            .map(|(&right, &step)| {
                if right {
                    step as isize
                } else {
                    -(step as isize)
                }
            })
            .sum()
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Node {
    Leaf,
    Branch { left: Box<Node>, right: Box<Node> },
}

impl Node {
    fn layout(&self, path: Vec<bool>, nodes: &mut Vec<PlacedNode>) -> usize {
        let index = nodes.len();
        nodes.push(PlacedNode {
            path: path.clone(),
            children: Vec::new(),
        });
        if let Node::Branch { left, right } = self {
            let mut left_path = path.clone();
            left_path.push(false);
            let mut right_path = path;
            right_path.push(true);
            let left = left.layout(left_path, nodes);
            let right = right.layout(right_path, nodes);
            nodes[index].children = vec![left, right];
        }
        index
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tree_generator_generates_leaf() {
        let mut choices = ChoiceSequence::new(vec![false]);

        let expected = Tree { root: Node::Leaf };

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
    }

    #[test]
    fn tree_generator_fails_for_incomplete_branch() {
        let mut choices = ChoiceSequence::new(vec![true, false]);

        let expected = None;

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

    #[test]
    fn tree_generator_tracks_draw_regions() {
        let mut choices = ChoiceSequence::new(vec![true, false, false]);
        let expected = vec![
            // left leaf
            (1, 2),
            // right leaf
            (2, 3),
            // whole tree
            (0, 3),
        ];

        Tree::generate(&mut choices);
        assert_eq!(choices.draws, expected);
    }

    #[test]
    fn tree_generator_does_not_track_invalid_draws() {
        let mut choices = ChoiceSequence::new(vec![true, false]);
        let expected = vec![
            // left leaf is successfully drawn, even though the tree itself fails to generate
            (1, 2),
        ];

        assert_eq!(Tree::generate(&mut choices), None);
        assert_eq!(choices.draws, expected);
    }
}
