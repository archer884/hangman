# Hangman

This project consists of three crates:

1. Hangman. This is basically just models used to communicate to and from the server; you can use these or ignore them.

2. Hangman server. This is the server. Run this to play the game. Unless you edit the code, it runs at localhost:8080.

3. Hangmanc lient. This is an example client and does almost nothing useful. It demonstrates basic communication with the server and implements a random solver and a manual solver. In most cases, the manual solver will be slightly more effective than the random solver.

## Hangman server

```shell
❯ hangman-server --help
hangman-server

USAGE:
    hangman-server <PATH>

ARGS:
    <PATH>    path to word list

OPTIONS:
    -h, --help    Print help information
```

The only option currently accepted by the server is a path to the word list. Words are selected from this list using a noise function seeded from God only knows what /dev/urandom nonsense on your system.

The only real way to understand the endpoints available is to examine the code, but basically:

- GET / new game
- GET /{game_uuid} read game state
- PUT /{game_uuid} update game state

An update to game state consists of the following json object:

```json
{
    "letter": "A"
}
```

There are probably a dozen better ways to do that, but I'm lazy. PRs welcome. :p
