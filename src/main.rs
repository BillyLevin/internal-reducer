use std::cmp;

#[derive(Debug)]
struct ChoiceSequence {
    choices: Vec<bool>,
    cursor: usize,
    draws: Vec<(usize, usize)>,
    draw_start_stack: Vec<usize>,
}

impl ChoiceSequence {
    fn new(choices: Vec<bool>) -> Self {
        Self {
            choices,
            cursor: 0,
            draws: Vec::new(),
            draw_start_stack: Vec::new(),
        }
    }

    fn next_choice(&mut self) -> Option<bool> {
        let item = self.choices.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(item)
    }

    fn prepare_replay(&mut self) {
        self.cursor = 0;
        self.draws.clear();
        self.draw_start_stack.clear();
    }

    fn draw<GeneratedItem>(
        &mut self,
        generate: impl FnOnce(&mut Self) -> Option<GeneratedItem>,
    ) -> Option<GeneratedItem> {
        self.draw_start_stack.push(self.cursor);

        let result = generate(self);
        let start = self.draw_start_stack.pop().expect("should not be empty");

        if result.is_some() {
            self.draws.push((start, self.cursor));
        }

        result
    }

    fn remove_unused_choices(&mut self) {
        self.choices.truncate(self.cursor);
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

/// Takes an "interesting" test-case and attempt to reduce it. Returns `Some` if the initial case
/// was interesting. In this case, the choices may or may not have been successfully reduced. Otherwise returns `None`
fn reduce<GeneratedItem>(
    choices: Vec<bool>,
    generate: impl Fn(&mut ChoiceSequence) -> Option<GeneratedItem>,
    is_interesting: impl Fn(&GeneratedItem) -> bool,
) -> Option<Vec<bool>> {
    let mut reduced = match evaluate(choices, &generate, &is_interesting) {
        Evaluation::Interesting(sequence) => sequence,
        Evaluation::Invalid | Evaluation::Uninteresting => return None,
    };

    while let Some(candidate) = zero_draw(&reduced, &generate, &is_interesting) {
        reduced = candidate;
    }

    Some(reduced.choices)
}

#[derive(Debug)]
enum Evaluation {
    Invalid,
    Interesting(ChoiceSequence),
    Uninteresting,
}

/// Attempts to generate an item and then evaluates whether it';s interesting
fn evaluate<GeneratedItem>(
    choices: Vec<bool>,
    generate: impl FnOnce(&mut ChoiceSequence) -> Option<GeneratedItem>,
    is_interesting: impl FnOnce(&GeneratedItem) -> bool,
) -> Evaluation {
    let mut choice_sequence = ChoiceSequence::new(choices);

    match generate(&mut choice_sequence) {
        Some(generated_item) => {
            choice_sequence.remove_unused_choices();

            if is_interesting(&generated_item) {
                Evaluation::Interesting(choice_sequence)
            } else {
                Evaluation::Uninteresting
            }
        }
        None => Evaluation::Invalid,
    }
}

fn zero_draw<GeneratedItem>(
    choices: &ChoiceSequence,
    generate: impl Fn(&mut ChoiceSequence) -> Option<GeneratedItem>,
    is_interesting: impl Fn(&GeneratedItem) -> bool,
) -> Option<ChoiceSequence> {
    for (region_start, region_end) in &choices.draws {
        let mut candidate_choices = choices.choices.clone();
        candidate_choices[*region_start..*region_end].fill(false);

        match evaluate(candidate_choices, &generate, &is_interesting) {
            Evaluation::Interesting(candidate)
                if shortlex_compare(&candidate.choices, &choices.choices)
                    == cmp::Ordering::Less =>
            {
                return Some(candidate);
            }
            Evaluation::Invalid | Evaluation::Uninteresting | Evaluation::Interesting(_) => {
                continue;
            }
        }
    }

    None
}

/// Orders two choice sequences. Priorities:
/// 1. length
/// 2. lexicographic (in this case `true` > `false`)
fn shortlex_compare(a: &[bool], b: &[bool]) -> cmp::Ordering {
    match a.len().cmp(&b.len()) {
        order @ (cmp::Ordering::Less | cmp::Ordering::Greater) => order,
        cmp::Ordering::Equal => a.cmp(b),
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reduction_works_for_interesting_cases() {
        let input = "10101010100".chars().map(|c| c == '1').collect();
        let expected: Vec<bool> = "1010100".chars().map(|c| c == '1').collect();

        let result = reduce(input, Tree::generate, Tree::has_height_imbalance)
            .expect("starting tree is interesting");

        assert_eq!(result, expected);

        assert!(matches!(
            evaluate(result, Tree::generate, Tree::has_height_imbalance),
            Evaluation::Interesting(_)
        ));
    }

    #[test]
    fn reduction_does_not_run_for_uninteresting_cases() {
        let input = "100".chars().map(|c| c == '1').collect();

        assert_eq!(
            reduce(input, Tree::generate, Tree::has_height_imbalance),
            None
        )
    }

    #[test]
    fn reduction_does_not_run_for_invalid_cases() {
        let input = "10".chars().map(|c| c == '1').collect();

        assert_eq!(
            reduce(input, Tree::generate, Tree::has_height_imbalance),
            None
        )
    }

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

        choices.prepare_replay();

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

        choices.prepare_replay();

        assert_eq!(Tree::generate(&mut choices), None);
        assert_eq!(choices.draws, expected);
    }

    #[test]
    fn shortlex_compare_works() {
        let cases = [
            ("111", "0000", cmp::Ordering::Less),
            ("0000", "111", cmp::Ordering::Greater),
            ("011", "101", cmp::Ordering::Less),
            ("101", "011", cmp::Ordering::Greater),
            ("1110", "1110", cmp::Ordering::Equal),
        ];

        for (bits_a, bits_b, expected) in cases {
            let choices_a: Vec<bool> = bits_a.chars().map(|c| c == '1').collect();
            let choices_b: Vec<bool> = bits_b.chars().map(|c| c == '1').collect();

            assert_eq!(shortlex_compare(&choices_a, &choices_b), expected);
        }
    }

    #[test]
    // LLM-generated
    fn zero_draw_finds_only_strict_interesting_improvements() {
        let cases = [
            ("1010100", None),              // Already minimal
            ("101010100", Some("1010100")), // Shorten a rightward chain
            ("111100000", Some("1110000")), // Shorten a leftward chain
        ];

        for (input, expected) in cases {
            let bits = input.chars().map(|c| c == '1').collect();
            let Evaluation::Interesting(current) =
                evaluate(bits, Tree::generate, Tree::has_height_imbalance)
            else {
                panic!("starting sequence must be interesting: {input}");
            };

            let result = zero_draw(&current, Tree::generate, Tree::has_height_imbalance);

            match (result, expected) {
                (None, None) => {}
                (Some(candidate), Some(expected)) => {
                    let expected_bits: Vec<bool> = expected.chars().map(|c| c == '1').collect();

                    assert_eq!(candidate.choices, expected_bits);
                    assert_eq!(candidate.cursor, candidate.choices.len());
                    assert!(candidate.draw_start_stack.is_empty());
                    assert_eq!(
                        shortlex_compare(&candidate.choices, &current.choices),
                        cmp::Ordering::Less,
                    );

                    let mut replay = ChoiceSequence::new(candidate.choices.clone());
                    let tree = Tree::generate(&mut replay).unwrap();
                    assert!(tree.has_height_imbalance());
                    assert_eq!(candidate.draws, replay.draws);
                }
                (result, expected) => {
                    panic!("input: {input}, expected: {expected:?}, got: {result:?}");
                }
            }
        }
    }
}
