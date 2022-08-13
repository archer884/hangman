use std::{cmp::Reverse, fs, io};

use hashbrown::{HashMap, HashSet};
use rand::seq::{IteratorRandom, SliceRandom};
use regex::Regex;
use squirrel_rng::SquirrelRng;

use super::Solver;

pub struct StrategicSolverFactory {
    dictionary: Vec<String>,
}

impl StrategicSolverFactory {
    pub fn from_path(dictionary: &str) -> io::Result<Self> {
        let text = fs::read_to_string(dictionary)?;
        Ok(Self::from_words(text.lines()))
    }

    fn from_words<'a>(words: impl Iterator<Item = &'a str>) -> Self {
        let mut dictionary: Vec<_> = words
            .filter_map(|word| {
                (word.len() >= 5 && word.is_ascii()).then(|| word.to_ascii_uppercase())
            })
            .collect();

        dictionary.sort_unstable();
        Self { dictionary }
    }

    #[allow(unused)]
    pub fn solver<'a>(&'a self) -> StrategicSolver<'a> {
        StrategicSolver {
            dictionary: &self.dictionary,
            ..Default::default()
        }
    }

    pub fn into_solver(self) -> IntoStrategicSolver {
        IntoStrategicSolver {
            dictionary: self.dictionary,
            ..Default::default()
        }
    }
}

#[derive(Debug, Default)]
pub struct IntoStrategicSolver {
    dictionary: Vec<String>,
    state: SolverState,
}

#[derive(Debug)]
struct SolverState {
    submitted: HashSet<u8>,
    uncharacterized: Option<u8>,
    disallow: HashSet<u8>,
    rng: SquirrelRng,
}

impl SolverState {
    fn next<T: AsRef<str>>(
        &mut self,
        word: &str,
        _guesses_remaining: usize,
        dictionary: &[T],
    ) -> char {
        println!("{word}");

        self.characterize(word);

        let expr = Shape {
            expr: build_expr(word).unwrap(),
            disallow: &self.disallow,
        };

        let filtered_dictionary = dictionary
            .iter()
            .map(|s| s.as_ref())
            .filter(|&text| expr.filter(text));

        let mut frequency = HashMap::new();
        for u in filtered_dictionary.flat_map(|word| word.bytes()) {
            *frequency.entry(u).or_insert(0usize) += 1;
        }

        let mut by_frequency: Vec<_> = frequency
            .into_iter()
            .filter(|entry| !self.submitted.contains(&entry.0))
            .collect();
        by_frequency.sort_unstable_by_key(|frequency| Reverse(frequency.1));

        let first_rank: Vec<_> = first_rank_by_key(by_frequency, |frequency| frequency.1)
            .map(|(value, _)| value)
            .collect();

        let selected = first_rank
            .choose(&mut self.rng)
            .map(|&u| u)
            .unwrap_or_else(|| (b'A'..=b'Z').choose(&mut self.rng).unwrap());

        self.uncharacterized = Some(selected);
        selected as char
    }

    fn characterize(&mut self, word: &str) {
        if let Some(u) = self.uncharacterized.take() {
            self.submitted.insert(u);
            if !word.bytes().any(|uword| u == uword) {
                self.disallow.insert(u);
            }
        }
    }
}

impl Default for SolverState {
    fn default() -> Self {
        Self {
            submitted: Default::default(),
            uncharacterized: Default::default(),
            disallow: Default::default(),

            // Chosen by mashing keyboard. Plenty random.
            rng: SquirrelRng::with_seed(3408509824),
        }
    }
}

fn first_rank_by_key<F, K, I: IntoIterator>(i: I, mut f: F) -> impl Iterator<Item = I::Item>
where
    F: FnMut(&I::Item) -> K,
    K: Eq,
{
    let mut key = None;
    i.into_iter()
        .filter_map(move |item| {
            let item_key = f(&item);
            if key.is_none() {
                key = Some(item_key);
                Some(item)
            } else if key == Some(item_key) {
                Some(item)
            } else {
                None
            }
        })
        .fuse()
}

impl Solver for IntoStrategicSolver {
    fn next_letter(&mut self, word: &str, guesses_remaining: usize) -> char {
        self.state.next(word, guesses_remaining, &self.dictionary)
    }
}

#[derive(Debug, Default)]
pub struct StrategicSolver<'a> {
    dictionary: &'a [String],
    state: SolverState,
}

impl Solver for StrategicSolver<'_> {
    fn next_letter(&mut self, word: &str, guesses_remaining: usize) -> char {
        self.state.next(word, guesses_remaining, &self.dictionary)
    }
}

struct Shape<'a> {
    expr: Regex,
    disallow: &'a HashSet<u8>,
}

impl Shape<'_> {
    fn filter(&self, text: &str) -> bool {
        self.expr.is_match(text) && text.bytes().all(|u| !self.disallow.contains(&u))
    }
}

fn build_expr(word: &str) -> Option<Regex> {
    let expr: String = word
        .bytes()
        .map(|u| match u {
            b'*' => b'.',
            u => u.to_ascii_uppercase(),
        } as char)
        .collect();
    Regex::new(&expr).ok()
}
