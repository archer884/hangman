mod random;
mod user;

pub use random::RandomSolver;
pub use user::UserInputSolver;

pub trait Solver {
    fn next_letter(&mut self, word: &str, guesses_remaining: usize) -> char;
}
