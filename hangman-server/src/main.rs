use std::{fs, io, sync::Mutex};

use actix_web::{
    web::{self, Data},
    App, HttpServer, Responder, ResponseError,
};
use clap::Parser;
use hangman::{CreateGameResponse, Game, UpdateGameRequest, UpdateGameResponse};
use hashbrown::{HashMap, HashSet};
use rand::{seq::SliceRandom, Rng};
use squirrel_rng::SquirrelRng;
use uuid::Uuid;

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, thiserror::Error)]
enum Error {
    #[error("game not found for id {0}")]
    GameNotFound(Uuid),
    #[error("guesses must consist of a single ASCII character - {0} is not valid")]
    IllegalGuess(String),
    #[error("guesses must be unique - {0} has already been guessed")]
    DuplicateGuess(String),
}

impl ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        actix_web::http::StatusCode::BAD_REQUEST
    }
}

#[derive(Clone, Debug, Parser)]
struct Args {
    /// path to word list
    path: String,
}

struct AppStateWithGameDb {
    shared: Mutex<(SquirrelRng, HashMap<Uuid, Game>)>,
    word_list: Vec<String>,
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
            .route("/", web::get().to(create_game))
            .route("/{game}", web::get().to(read_game))
            .route("/{game}", web::put().to(update_game))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

async fn create_game(data: Data<AppStateWithGameDb>) -> io::Result<impl Responder> {
    let mut state = data.shared.lock().unwrap();

    let game = build_game(&data.word_list, &mut state.0);
    let id = Uuid::new_v4();

    state.1.insert(id.clone(), game.clone());

    let response = CreateGameResponse::new(id, &game);

    Ok(web::Json(response))
}

async fn read_game(id: web::Path<Uuid>, data: Data<AppStateWithGameDb>) -> Result<impl Responder> {
    let state = data.shared.lock().unwrap();
    let game = state
        .1
        .get(&id)
        .ok_or_else(|| Error::GameNotFound(id.into_inner()))?;

    if game.is_lost() {
        return Ok(web::Json(UpdateGameResponse::lose(
            &game.word,
            "Better luck next time!",
        )));
    }

    if game.is_won() {
        return Ok(web::Json(UpdateGameResponse::win(
            &game.word,
            "I said you won! Stop rubbing it in. >.<",
        )));
    }

    Ok(web::Json(UpdateGameResponse::update(game)))
}

async fn update_game(
    id: web::Path<Uuid>,
    request: web::Json<UpdateGameRequest>,
    data: Data<AppStateWithGameDb>,
) -> Result<impl Responder> {
    // First, let's validate that the update even looks kind of legal. It can only be legal if
    // it is one character in length and all ascii.

    let UpdateGameRequest { letter } = request.into_inner();

    if letter.len() != 1 || !letter.is_ascii() {
        return Err(Error::IllegalGuess(letter));
    }

    // Next, we'll grab the uuid for the game from the route and see if we can find a matching
    // game state. If we fail at any of this, we're just going to tell the user we can't find his
    // game. I mean, it's his job to keep up with these darned things. He's lucky we don't
    // pretend to find it and then throw away one of his socks.

    let mut state = data.shared.lock().unwrap();
    let game = state
        .1
        .get_mut(&id)
        .ok_or_else(|| Error::GameNotFound(id.into_inner()))?;

    // A game continuation isn't relevant if the game has been won or lost, so we'll check to
    // see if the game is over yet.

    if game.is_lost() {
        return Ok(web::Json(UpdateGameResponse::lose(
            &game.word,
            "Better luck next time!",
        )));
    }

    if game.is_won() {
        return Ok(web::Json(UpdateGameResponse::win(
            &game.word,
            "I said you won! Stop rubbing it in. >.<",
        )));
    }

    // Now that all that is out of the way, we can go about examining the guess itself. It must
    // not duplicate anything we've already seen.

    let guess = letter.bytes().next().unwrap().to_ascii_uppercase();

    if game.correct.contains(&guess) || game.incorrect.contains(&guess) {
        return Err(Error::DuplicateGuess(letter));
    }

    // Now, if the guess matches any character in the word, we will add that guess to the
    // "correct" set. We'll then check again to see if the game has been won and respond
    // accordingly.

    // If instead the guess fails to match anything, we add it to the "incorrect" set and check
    // to see whether the user has just LOST the game. In that case, we will send him a nastygram
    // and log this in his permanent file.

    if game.word.bytes().any(|u| u == guess) {
        game.correct.insert(guess);
        if game.is_won() {
            if game.guesses_remaining() >= 3 {
                return Ok(web::Json(UpdateGameResponse::win(
                    &game.word,
                    "FLAWLESS VICTORY!",
                )));
            } else {
                return Ok(web::Json(UpdateGameResponse::win(
                    &game.word,
                    "Victory is yours!",
                )));
            }
        }

        Ok(web::Json(UpdateGameResponse::update(game)))
    } else {
        game.incorrect.insert(guess);
        if game.is_lost() {
            return Ok(web::Json(UpdateGameResponse::lose(
                &game.word,
                "Sorry, friend. You've been hanged!",
            )));
        }

        Ok(web::Json(UpdateGameResponse::update(game)))
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
