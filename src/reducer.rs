use std::cmp;

#[derive(Debug, hegel::PrettyPrintable)]
pub struct ChoiceSequence {
    pub choices: Vec<bool>,
    pub cursor: usize,
    pub draws: Vec<(usize, usize)>,
    pub draw_start_stack: Vec<usize>,
}

impl ChoiceSequence {
    pub fn new(choices: Vec<bool>) -> Self {
        Self {
            choices,
            cursor: 0,
            draws: Vec::new(),
            draw_start_stack: Vec::new(),
        }
    }

    pub fn next_choice(&mut self) -> Option<bool> {
        let item = self.choices.get(self.cursor).copied()?;
        self.cursor += 1;
        Some(item)
    }

    pub fn draw<GeneratedItem>(
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

/// Takes an "interesting" test-case and attempt to reduce it. Returns `Some` if the initial case
/// was interesting. In this case, the choices may or may not have been successfully reduced. Otherwise returns `None`
pub fn reduce<GeneratedItem>(
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

#[cfg(test)]
mod tests {
    use hegel::{TestCase, generators};

    use crate::tree::Tree;

    use super::*;

    #[hegel::test(test_cases = 1000)]
    fn reduction_preserves_interestingness(test_case: TestCase) {
        let initial_choices = test_case.draw(generators::vecs(generators::booleans()));

        let is_interesting = Tree::generate(&mut ChoiceSequence::new(initial_choices.clone()))
            .is_some_and(|tree| tree.has_height_imbalance());

        let reduced = reduce(
            initial_choices.clone(),
            Tree::generate,
            Tree::has_height_imbalance,
        );

        if is_interesting {
            assert!(reduced.is_some_and(|choices| {
                let is_still_interesting =
                    Tree::generate(&mut ChoiceSequence::new(choices.clone()))
                        .is_some_and(|tree| tree.has_height_imbalance());

                let is_not_shortlex_larger = matches!(
                    shortlex_compare(&choices, &initial_choices),
                    cmp::Ordering::Less | cmp::Ordering::Equal
                );

                is_still_interesting && is_not_shortlex_larger
            }));
        } else {
            assert_eq!(reduced, None)
        }
    }

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
}
