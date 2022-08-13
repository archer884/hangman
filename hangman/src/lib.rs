use std::borrow::Cow;

use hashbrown::HashSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Game {
    pub word: String,
    pub correct: HashSet<u8>,
    pub incorrect: HashSet<u8>,
}

impl Game {
    pub fn is_lost(&self) -> bool {
        (7 - self.incorrect.len() as i32) <= 0
    }

    pub fn is_won(&self) -> bool {
        self.word.bytes().all(|u| self.correct.contains(&u))
    }

    pub fn masked_word(&self) -> String {
        self.word
            .bytes()
            .map(|u| if self.correct.contains(&u) { u } else { b'*' } as char)
            .collect()
    }

    pub fn guesses_remaining(&self) -> i32 {
        (7i32 - self.incorrect.len() as i32).max(0)
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateGameResponse {
    pub id: Uuid,
    pub word: String,
    pub guesses: i32,
}

impl CreateGameResponse {
    pub fn new(id: Uuid, game: &Game) -> Self {
        Self {
            id,
            word: game.masked_word(),
            guesses: game.guesses_remaining(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct GameResponse {
    pub word: String,
    pub guesses: i32,
}

impl GameResponse {
    fn new(game: &Game) -> Self {
        Self {
            word: game.masked_word(),
            guesses: game.guesses_remaining(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct UpdateGameRequest {
    pub letter: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(untagged)]
pub enum UpdateGameResponse {
    Update(GameResponse),
    Finalize {
        victory: bool,
        message: Cow<'static, str>,
        word: String,
    },
}

impl UpdateGameResponse {
    pub fn update(game: &Game) -> Self {
        UpdateGameResponse::Update(GameResponse::new(game))
    }

    pub fn win(word: impl Into<String>, message: &'static str) -> Self {
        UpdateGameResponse::Finalize {
            victory: true,
            message: message.into(),
            word: word.into(),
        }
    }

    pub fn lose(word: impl Into<String>, message: &'static str) -> Self {
        UpdateGameResponse::Finalize {
            victory: false,
            message: message.into(),
            word: word.into(),
        }
    }
}
