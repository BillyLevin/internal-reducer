pub mod reducer;
pub mod tree;

use std::env;

use crate::{
    reducer::{ChoiceSequence, reduce},
    tree::Tree,
};

fn main() {
    let args: Vec<String> = env::args().collect();

    let Some(input) = args.get(1) else {
        eprintln!("please provide a choice sequence");
        return;
    };

    let choices: Vec<bool> = input.chars().map(|c| c == '1').collect();

    let Some(initial_tree) = Tree::generate(&mut ChoiceSequence::new(choices.clone())) else {
        eprintln!("invalid choice sequence for generating trees");
        return;
    };

    println!("initial tree: {input}\n{initial_tree}");

    let reduced = reduce(choices, Tree::generate, Tree::has_height_imbalance);

    match reduced.and_then(|reduced_choices| {
        let choice_string = reduced_choices
            .iter()
            .map(|&choice| if choice { '1' } else { '0' })
            .collect::<String>();

        Tree::generate(&mut ChoiceSequence::new(reduced_choices)).map(|tree| (choice_string, tree))
    }) {
        Some((reduced_choices, reduced_tree)) => {
            println!("\nreduced tree: {reduced_choices}\n{reduced_tree}")
        }
        None => println!("\ntree could not be reduced"),
    };
}
