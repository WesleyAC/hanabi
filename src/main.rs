#![feature(vec_remove_item)]
#![feature(proc_macro_hygiene)]
#![feature(decl_macro)]

#[macro_use] extern crate rocket;

use uuid::Uuid;

use rand::thread_rng;
use rand::seq::SliceRandom;

use serde::{Deserialize, Serialize};

use rocket::State;
use rocket::http::RawStr;
use rocket::response::content::Html;

use rocket_contrib::json::Json;
use rocket_contrib::serve::StaticFiles;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

#[derive(Clone, Debug, Hash, PartialEq, Eq, Serialize, Deserialize)]
enum Color {
    Red,
    Green,
    Blue,
    White,
    Yellow,
}

type Number = u8;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
struct Card {
    uuid: Uuid,
    number: Number,
    color: Color,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum HintData {
    Color(Color),
    Number(Number),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Hint {
    player: Player,
    data: HintData,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Hand {
    cards: Vec<Card>,
}

type Player = usize;

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Game {
    player_names: Vec<String>,
    players: Vec<Hand>,
    deck: Vec<Card>,
    discard: Vec<Card>,
    played: HashMap<Color, Option<u8>>,
    given_hints: HashMap<Uuid, Vec<HintData>>,
    hints: u8,
    fuses: u8,
    turn: Player,
    endgame_turns: usize,
    moves: Vec<PlayerTurn>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
enum Turn {
    Play(Card),
    Hint(Hint),
    Discard(Card),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct PlayerTurn {
    player: Player,
    turn: Turn,
}

impl Game {
    fn new(num_players: usize) -> Self {
        let mut deck = vec![];
        for color in [Color::Red, Color::Green, Color::Blue, Color::White, Color::Yellow].iter() {
            for number in 1..=5 {
                let n = match number {
                    1 => 3,
                    2..=4 => 2,
                    5 => 1,
                    _ => unreachable!(),
                };
                for _ in 0..n {
                    deck.push(Card {
                        uuid: Uuid::new_v4(),
                        color: color.clone(),
                        number: number.clone(),
                    });
                }
            }
        }
        deck.shuffle(&mut thread_rng());

        let mut players = vec![];
        for _ in 0..num_players {
            let mut cards = vec![];
            let num_cards = match num_players {
                2..=3 => 5,
                4..=5 => 4,
                _ => unreachable!(),
            };
            for _ in 0..num_cards {
                cards.push(deck.pop().unwrap());
            }
            players.push(Hand { cards });
        }

        Game {
            player_names: vec![],
            players,
            deck,
            discard: vec![],
            played: HashMap::new(),
            given_hints: HashMap::new(),
            hints: 8,
            fuses: 3,
            turn: 0,
            endgame_turns: num_players + 1, // do i need +1? or +2?
            moves: vec![],
        }
    }
}

fn play_turn(game: &Game, turn: &PlayerTurn) -> Option<Game> {
    if game.fuses == 0 { return None; }
    if turn.player >= game.players.len() ||
        (game.deck.len() == 0 && game.endgame_turns == 0) {
        return None;
    }
    let mut game = game.clone();
    match &turn.turn {
        Turn::Play(card) => {
            let card = game.players[turn.player].cards.remove_item(&card)?;
            let current = game.played.get(&card.color).unwrap_or(&None).unwrap_or(0);
            if card.number == current + 1 {
                if card.number == 5 {
                    game.hints = std::cmp::min(game.hints + 1, 8);
                }
                game.played.insert(card.color, Some(card.number));
            } else {
                game.discard.push(card);
                game.fuses -= 1;
            }
            if let Some(new_card) = game.deck.pop() {
                game.players[turn.player].cards.push(new_card);
            }
        },
        Turn::Hint(hint) => {
            if game.hints == 0 { return None; }
            if hint.player == turn.player { return None; }
            let mut cardhints = game.players[hint.player].cards.iter().filter_map(|card| {
                match &hint.data {
                    HintData::Color(color) => {
                        if color == &card.color {
                            Some((card.uuid, HintData::Color(color.clone())))
                        } else {
                            None
                        }
                    },
                    HintData::Number(number) => {
                        if number == &card.number {
                            Some((card.uuid, HintData::Number(number.clone())))
                        } else {
                            None
                        }
                    },
                }
            }).peekable();
            if cardhints.peek().is_none() { return None; }
            for (card, hintdata) in cardhints {
                if game.given_hints.get(&card).is_none() {
                    game.given_hints.insert(card.clone(), vec![]);
                }
                game.given_hints.get_mut(&card).unwrap().push(hintdata);
            }
            game.hints = std::cmp::max(game.hints - 1, 0);
        },
        Turn::Discard(card) => {
            let card = game.players[turn.player].cards.remove_item(&card)?;
            game.discard.push(card);
            if let Some(new_card) = game.deck.pop() {
                game.players[turn.player].cards.push(new_card);
            }
            game.hints = std::cmp::min(game.hints + 1, 8);
        },
    }
    game.turn = (game.turn + 1) % game.players.len();
    if game.deck.len() == 0 {
        game.endgame_turns -= 1;
    }
    game.moves.push(turn.clone());
    Some(game)
}

#[derive(Deserialize, Serialize)]
struct GameSetup {
    name: String,
    players: usize,
}

#[post("/newgame", data = "<setup>")]
fn newgame(state: State<ServerState>, setup: Json<GameSetup>) -> Option<()> {
    let mut games = state.inner().games.lock().unwrap();
    if games.get(&setup.name).is_none() {
        games.insert(setup.name.clone(), Arc::new(Mutex::new(Game::new(setup.players))));
        Some(())
    } else {
        None
    }
}

#[get("/<game>/gamedata")]
fn gamedata(game: &RawStr, state: State<ServerState>) -> Option<Json<Game>> {
    Some(Json(state.inner().games.lock().unwrap().get(&game.to_string())?.lock().unwrap().clone()))
}

#[post("/<game>/join/<name>")]
fn join(game: &RawStr, name: &RawStr, state: State<ServerState>) -> Option<()> {
    let games = state.inner().games.lock().unwrap();
    let mut game = games.get(&game.to_string())?.lock().unwrap();
    if !game.player_names.iter().any(|x| x == &name.to_string()) && game.player_names.len() < game.players.len() {
        game.player_names.push(name.to_string());
    }
    Some(())
}

#[post("/<game>/play", data = "<turn>")]
fn play(game: &RawStr, state: State<ServerState>, turn: Json<PlayerTurn>) -> Option<()> {
    let games = state.inner().games.lock().unwrap();
    let mut game = games.get(&game.to_string())?.lock().unwrap();
    if let Some(new_game) = play_turn(&game, &turn.into_inner()) {
        *game = new_game
    }
    Some(())
}

#[get("/<_game>")]
fn gameindex(_game: &RawStr) -> Html<&str> {
    Html(include_str!("../static/game.html"))
}

// TODO: figure out what is actually going on here and stop throwing mutexes at the problem
struct ServerState {
    games: Arc<Mutex<HashMap<String, Arc<Mutex<Game>>>>>,
}

fn main() {
    let state = ServerState {
        games: Arc::new(Mutex::new(HashMap::new())),
    };
    rocket::ignite()
        .mount("/", StaticFiles::from("./static"))
        .mount("/api/", routes![newgame, gamedata, join, play])
        .mount("/game/", routes![gameindex])
        .manage(state)
        .launch();
}
