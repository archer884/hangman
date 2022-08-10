use std::{fs, io, sync::Mutex};

use actix_web::{
    web::{self, Data},
    App, HttpServer, Responder,
};
use clap::Parser;
use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom, Rng};
use serde::{Deserialize, Serialize};
use squirrel_rng::SquirrelRng;
use uuid::Uuid;

#[derive(Clone, Debug, Parser)]
struct Args {
    /// path to word list
    path: String,
}

struct AppStateWithGameDb {
    shared: Mutex<(SquirrelRng, HashMap<Uuid, Game>)>,
    word_list: Vec<String>,
}

#[derive(Clone, Debug)]
struct Game {
    word: String,
    correct: HashSet<u8>,
    incorrect: HashSet<u8>,
}

impl Game {
    fn is_lost(&self) -> bool {
        (7 - self.incorrect.len() as i32) <= 0
    }

    fn is_won(&self) -> bool {
        self.word.bytes().all(|u| self.correct.contains(&u))
    }

    fn masked_word(&self) -> String {
        self.word
            .bytes()
            .map(|u| if self.correct.contains(&u) { u } else { b'*' } as char)
            .collect()
    }

    fn guesses_remaining(&self) -> i32 {
        (7i32 - self.incorrect.len() as i32).max(0)
    }
}

#[derive(Clone, Debug, Serialize)]
struct GameResponse {
    id: Uuid,
    word: String,
    guesses: i32,
}

#[derive(Clone, Debug, Deserialize)]
struct GameRequest {
    id: Uuid,
    letter: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(untagged)]
enum PlayResponse {
    Victory { message: &'static str },
    Defeat { message: &'static str },
    Illegal { message: &'static str },
    Continue(GameResponse),
}

// FIXME: There is no reason whatsoever to use an async framework on this project. >.<

#[actix_web::main]
async fn main() -> io::Result<()> {
    let args = Args::parse();

    let data = Data::new(AppStateWithGameDb {
        shared: Mutex::new((SquirrelRng::new(), HashMap::new())),
        word_list: read_words(&args.path)?,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/", web::get().to(new_game))
            .route("/", web::put().to(play_game))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn new_game(data: Data<AppStateWithGameDb>) -> io::Result<impl Responder> {
    let mut state = data.shared.lock().expect("don't poison my lock, ok?");

    let game = build_game(&data.word_list, &mut state.0);
    let id = Uuid::new_v4();

    state.1.insert(id.clone(), game.clone());

    let response = GameResponse {
        id,
        word: game.masked_word(),
        guesses: game.guesses_remaining(),
    };

    Ok(web::Json(response))
}

async fn play_game(
    request: web::Json<GameRequest>,
    data: Data<AppStateWithGameDb>,
) -> io::Result<impl Responder> {
    let mut state = data.shared.lock().expect("I said DON'T poison it!");

    let game = state.1.get_mut(&request.id).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no known game with id({})", request.id),
        )
    })?;

    if game.is_lost() {
        return Ok(web::Json(PlayResponse::Defeat {
            message: "Better luck next time!",
        }));
    }

    if game.is_won() {
        return Ok(web::Json(PlayResponse::Victory {
            message: "I said you won! Stop rubbing it in. >.<",
        }));
    }

    if request.letter.len() != 1 {
        return Ok(web::Json(PlayResponse::Illegal {
            message: "Your guess must consist of a single letter.",
        }));
    }

    let guess = request
        .letter
        .bytes()
        .next()
        .expect("We just went over this...")
        .to_ascii_uppercase();

    if game.correct.contains(&guess) {
        return Ok(web::Json(PlayResponse::Illegal {
            message: "Your guesses must be unique.",
        }));
    }

    if game.word.bytes().any(|u| u == guess) {
        game.correct.insert(guess);
        if game.is_won() {
            if game.guesses_remaining() >= 3 {
                return Ok(web::Json(PlayResponse::Victory {
                    message: "FLAWLESS VICTORY!",
                }));
            } else {
                return Ok(web::Json(PlayResponse::Victory {
                    message: "Victory is yours!",
                }));
            }
        }

        Ok(web::Json(PlayResponse::Continue(GameResponse {
            id: request.id,
            word: game.masked_word(),
            guesses: game.guesses_remaining(),
        })))
    } else {
        game.incorrect.insert(guess);
        if game.is_lost() {
            return Ok(web::Json(PlayResponse::Defeat {
                message: "Sorry, friend. You've been hanged!",
            }));
        }

        Ok(web::Json(PlayResponse::Continue(GameResponse {
            id: request.id,
            word: game.masked_word(),
            guesses: game.guesses_remaining(),
        })))
    }
}

fn build_game(words: &[String], rng: &mut impl Rng) -> Game {
    Game {
        word: words
            .choose(rng)
            .expect("your word list is empty!")
            .to_owned(),
        correct: HashSet::new(),
        incorrect: HashSet::new(),
    }
}

fn read_words(path: &str) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(path)?;
    let words = text
        .lines()
        .filter(|&word| word.is_ascii() && word.len() >= 5)
        .map(|word| word.to_ascii_uppercase())
        .collect();
    Ok(words)
}
