use std::{io, process};

use clap::{Parser, Subcommand};
use hangman::{CreateGameResponse, UpdateGameRequest, UpdateGameResponse};
use reqwest::{blocking::Client, StatusCode};
use solver::{RandomSolver, Solver, StrategicSolverFactory, UserInputSolver};

mod solver;

#[derive(Debug, Parser)]
struct Args {
    /// hangman url
    server: String,

    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand)]
enum Command {
    Random,
    Strategic(SolverConfig),
    User,
}

#[derive(Debug, Parser)]
struct SolverConfig {
    /// path to dictionary
    dictionary: String,
}

fn main() {
    if let Err(e) = run(&Args::parse()) {
        eprintln!("{e}");
        process::exit(1);
    }
}

fn run(args: &Args) -> anyhow::Result<()> {
    let client = Client::builder()
        .user_agent(concat!("hangman-client v", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap();

    let CreateGameResponse { id, word, guesses } = client.get(&args.server).send()?.json()?;
    let game_url = format!("{}/{}", args.server, id);

    let mut word = word;
    let mut guesses_remaining = guesses as usize;
    let mut solver = build_solver(&args.command)?;

    loop {
        let letter = solver.next_letter(&word, guesses_remaining).to_string();
        let update = client.put(&game_url).json(&UpdateGameRequest { letter });

        let update: UpdateGameResponse = match update.send() {
            Ok(response) => response.json()?,
            Err(e) if e.status() == Some(StatusCode::BAD_REQUEST) => {
                // The most likely reason for this error is a bad request, which we don't really
                // care about. It's probably just the server whining that you already guessed the
                // letter C.
                continue;
            }
            Err(e) => return Err(e.into()),
        };

        match update {
            UpdateGameResponse::Finalize {
                victory,
                message,
                word,
            } => {
                if victory {
                    println!("The word was: {word}\nWe win! {message}");
                } else {
                    println!("The word was: {word}\nWe lose. :( {message}");
                }
                break;
            }
            UpdateGameResponse::Update(update) => {
                word = update.word;
                guesses_remaining = update.guesses as usize;
            }
        }
    }

    Ok(())
}

fn build_solver(command: &Command) -> io::Result<Box<dyn Solver>> {
    match command {
        Command::Random => Ok(Box::new(RandomSolver::new())),
        Command::Strategic(config) => Ok(Box::new(
            StrategicSolverFactory::from_path(&config.dictionary)?.into_solver(),
        )),
        Command::User => Ok(Box::new(UserInputSolver)),
    }
}
