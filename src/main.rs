use rand::seq::SliceRandom;
use std::collections::VecDeque;
use std::fs::File;
use std::io::{self, BufRead, BufReader};
use std::path::Path;
use std::sync::{Arc, Mutex};

#[derive(Debug, PartialEq, Eq)]
struct Card(usize);

#[derive(Debug)]
struct Deck {
    number: usize,
    cards: VecDeque<Card>,
}

#[derive(Debug)]
struct Player<'a> {
    number: usize,
    draw_deck: &'a Arc<Mutex<Deck>>,
    discard_deck: &'a Arc<Mutex<Deck>>,
    hand: Vec<Card>,
}

impl<'a> Player<'a> {
    fn has_winning_hand(&self) -> bool {
        let winning = self.hand.windows(2).all(|w| w[0] == w[1]);
        if winning {
            println!("Player {} has won! 🥳  with hand {:?}", self.number, self.hand);
        }
        return winning;
    }

    fn select_discard_card(&self) -> Option<usize> {
        let possibles = self
            .hand
            .iter()
            .filter(|&c| c.0 != self.number)
            .collect::<Vec<&Card>>();

        match possibles.choose(&mut rand::thread_rng()).copied() {
            Some(chosen) => self.hand.iter().position(|c| c == chosen),
            None => None,
        }
    }

    fn take_turn(&mut self) {
        let mut draw_deck = self.draw_deck.lock().unwrap();
        let new_card = match draw_deck.cards.pop_front() {
            Some(nc) => nc,
            None => {
                println!("Player {}'s draw deck is empty!", self.number);
                return;
            } // draw deck is empty, end turn
        };
        println!(
            "Player {} drawns a {} from Deck {}",
            self.number, new_card.0, draw_deck.number
        );

        let discard_card = match self.select_discard_card() {
            Some(discard_index) => {
                let discard = self.hand.remove(discard_index);
                self.hand.push(new_card);
                discard
            }
            None => new_card,
        };

        let mut discard_deck = self.discard_deck.lock().unwrap();
        println!(
            "Player {} discards {} to Deck {}",
            self.number, discard_card.0, discard_deck.number
        );
        discard_deck.cards.push_back(discard_card);
    }
}

fn main() {
    let n: usize = loop {
        match get_n() {
            Ok(n) => break n,
            Err(e) => println!("{e}"),
        }
    };

    let mut pack = loop {
        match get_pack(&n) {
            Ok(pack) => break pack,
            Err(e) => println!("{e}"),
        }
    };

    let mut decks: Vec<Deck> = (1..=n)
        .map(|i| Deck {
            number: i,
            cards: VecDeque::new(),
        })
        .collect();

    // Deal cards to decks
    for i in (4 * n)..(8 * n) {
        decks[i % n].cards.push_front(pack.remove(4 * n));
    }

    let decks: Vec<Arc<Mutex<Deck>>> = decks.into_iter().map(|d| Arc::new(Mutex::new(d))).collect();

    let mut players: Vec<Player> = (1..=n)
        .map(|i| Player {
            number: i,
            draw_deck: &decks[i - 1],
            discard_deck: &decks[(i) % n],
            hand: Vec::new(),
        })
        .collect();

    // Deal cards to players
    for i in (0..4 * n).rev() {
        players[i % n].hand.push(
            pack.pop()
                .expect("Pack is not full enough! Probably an index error."),
        );
    }

    'game: loop {
        for player in &mut players {
            if player.has_winning_hand() {
                break 'game;
            }
            player.take_turn();
            if player.has_winning_hand() {
                break 'game;
            }
        }
    }
}

fn get_n() -> Result<usize, String> {
    println!("Please enter the number of players:");
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|e| e.to_string())?;
    let i = input
        .trim()
        .parse::<usize>()
        .map_err(|e| format!("The number of players must be a positive integer! {e}"))?;
    if i < 1 {
        return Err(format!(
            "The game must have a non-zero number of players, but was {}!",
            i
        ));
    }
    return Ok(i);
}

fn get_pack(n: &usize) -> Result<Vec<Card>, String> {
    println!("Please enter the location of the pack to load:");
    let mut path_str = String::new();
    io::stdin()
        .read_line(&mut path_str)
        .map_err(|e| e.to_string())?;
    path_str = path_str
        .trim()
        .parse::<String>()
        .map_err(|e| format!("Could not parse input file name string! {}.", e))?;
    let path = Path::new(&path_str);
    let file = File::open(&path)
        .map_err(|e| format!("Could not open file {}! Because {}.", path.display(), e))?;

    let reader = BufReader::new(&file);
    let mut pack = Vec::new();

    for (i, line) in reader.lines().enumerate() {
        let line = line.map_err(|e| format!("Could not read line {}! Reason: {}.", i + 1, e))?;

        let val = line.parse::<usize>().map_err(|_| {
            format!(
                "Could not parse \"{}\" on line {} as a possitive integer!",
                line,
                i + 1
            )
        })?;
        pack.push(Card(val))
    }

    if pack.len() != 8 * n {
        return Err(format!(
            "A decks must have 8n ({}) cards, but the supplied deck had {}.",
            8 * n,
            pack.len()
        ));
    }
    return Ok(pack);
}
