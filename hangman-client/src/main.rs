use std::{fs, io, process};

use clap::{Parser, Subcommand};
use hangman::{CreateGameResponse, UpdateGameRequest, UpdateGameResponse};
use reqwest::blocking::Client;
use solver::{RandomSolver, Solver, UserInputSolver};

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
    let mut solver = build_solver(&args.command)?;
    let client = Client::builder()
        .user_agent(concat!("hangman-client v", env!("CARGO_PKG_VERSION")))
        .build()
        .unwrap();

    let CreateGameResponse { id, word, guesses } = client.get(&args.server).send()?.json()?;
    let game_url = format!("{}/{}", args.server, id);

    let mut word = word;
    let mut guesses_remaining = guesses as usize;

    loop {
        let letter = solver.next_letter(&word, guesses_remaining).to_string();
        let update: UpdateGameResponse = client
            .put(&game_url)
            .json(&UpdateGameRequest { letter })
            .send()?
            .json()?;

        match update {
            UpdateGameResponse::Finalize { victory, message } => {
                if victory {
                    println!("We win! {message}");
                } else {
                    println!("We lose. :( {message}");
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
        Command::Strategic(_dictionary) => {
            todo!()
        }
        Command::User => Ok(Box::new(UserInputSolver)),
    }
}

fn read_dictionary(dictionary: &str) -> io::Result<Vec<String>> {
    let text = fs::read_to_string(dictionary)?;
    Ok(text.lines().map(ToOwned::to_owned).collect())
}
