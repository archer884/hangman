use rand::seq::SliceRandom;
use squirrel_rng::SquirrelRng;

use super::Solver;

pub struct RandomSolver {
    idx: usize,
    alpha: Vec<u8>,
}

impl RandomSolver {
    pub fn new() -> Self {
        let mut alpha: Vec<_> = b"abcdefghijklmnopqrstuvwxyz".iter().copied().collect();
        alpha.shuffle(&mut SquirrelRng::new());
        Self { idx: 0, alpha }
    }

    fn next(&mut self) -> char {
        if self.idx >= self.alpha.len() {
            self.idx = 0;
        }

        let next = self.alpha[self.idx];
        self.idx += 1;
        next as char
    }
}

impl Solver for RandomSolver {
    fn next_letter(&mut self, word: &str, _guesses_remaining: usize) -> char {
        println!("{word}");
        self.next()
    }
}
