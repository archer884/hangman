use read_input::{shortcut::input, InputBuild};

use super::Solver;

pub struct UserInputSolver;

impl Solver for UserInputSolver {
    fn next_letter(&mut self, word: &str, guesses_remaining: usize) -> char {
        println!("{word} (Guesses remaining: {guesses_remaining}");
        input()
            .msg("Guess: ")
            .repeat_msg("Guess: ")
            .err("Try entering just one character")
            .get()
    }
}
