extern crate term;

use::std::env;

//TODO: make BOARD_LENGTH dynamic
const BOARD_LENGTH: usize = 25;

#[derive(Debug, Clone)]
/// Defines the board/tape on which the busy beaver game will unfold
/// The board in theory can be of inifinte length. In the real world
/// we are limited by RAM.
struct Board(Vec<u8>);

#[derive(Debug, Clone, Copy)]
/// Directions the 'typewriter' (tw) can move.
enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
/// The choosen symbols for our tw that can be written on the tape
struct Symbol(u8);

#[derive(Debug, Clone, Copy, PartialEq)]
/// States of the the Turing Machine
enum State {
    Card(usize),
    Halt,
}

#[derive(Debug, Clone)]
/// Defines a card holding instructions.
struct Cards(Vec<Card>);

#[derive(Debug, Clone)]
/// Defines a card holding instructions.
struct Card(Vec<Instructions>);

#[derive(Debug, Clone, Copy)]
/// Holds the instruction the tw must follow
struct Instructions {
    /// Symbol to be written on the board
    write: Symbol,

    /// The direction the tw moves after write
    direction: Direction,

    /// State of next operation
    state: State,
}

/// Typewriter holds and controls the posiston of the moving head 
struct Typewriter {
    head: usize,
}

#[derive(Debug)]
/// Holds all the generated machines
struct Machines(Vec<Cards>);

#[derive(Debug, Clone)]
/// Keep track of stats during a run of busy beaver
struct Stats {
    cards:          Cards,
    score:          usize,
    action:         usize,
    boards:         Vec<Board>,
    head_positions: Vec<usize>,
}

impl Instructions {
    pub fn new(write: Symbol, direction: Direction, state: State) -> Self {
        Self { write, direction, state }
    }
}

impl Card {
    pub fn new(instructions: Vec<Instructions>) -> Self {
        Self(instructions)
    }

    /// fetch the instructions corresponding to the read symbol
    /// on the board.
    pub fn get_instruction<'a>(
        &'a self, tw: &Typewriter, board: &Board) -> &'a Instructions {
        match tw.read(board) {
            Symbol(i) => &self.0[i as usize],
        }
    }
}

impl Cards {
    pub fn new(card_vec: Vec<Card>) -> Self {
        Self(card_vec)
    }

    pub fn get_card(&self, ind: usize) -> &Card {
        &self.0[ind]
    }

    pub fn get_states(&self) -> Vec<State> {
        let mut states = Vec::new();
        for card in self.0.iter() {
            let (inst1, inst2) = (card.0[0], card.0[1]);
            states.extend(vec![inst1.state,inst2.state]);
        }
        states
    }

    pub fn contains_halt_state(&self) -> bool {
        self.get_states().contains(&State::Halt)
    }
}

impl Typewriter {
    /// Move the head of the typewriter one step ether left or right.
    pub fn move_head(&mut self, dir: &Direction) ->  bool {
        if !(self.head > 0 && self.head < (BOARD_LENGTH-1)) {
            return false
        }

        match dir {
            Direction::Left  => self.head = self.head - 1,
            Direction::Right => self.head = self.head + 1,
        }
        true
    }

    /// Writes the new symbol onto the board.
    pub fn write(&self, sym: &Symbol, board: &mut Board) {
        match sym {
            Symbol(i) => board.0[self.head] = *i,
        }
    }

    /// Reads symbol of the board of which the tw head is hovering.
    pub fn read(&self, board: &Board) -> Symbol {
        Symbol(board.0[self.head])
    }
}

impl Machines {
    pub fn generate(num_cards: usize, num_symbols: usize) -> Self {
        // construct every possible set of instructions
        let inst = Self::generate_instructions(num_cards, num_symbols);

        // constructs all possible cards
        let cards = Self::generate_all_possible_cards(inst, num_symbols);

        // constructs all possible machines that contains the halt state
        let num_configs = 
            (num_symbols*2*(num_cards+1)).pow((num_cards+num_symbols) as u32);

        println!("Starts creating machines");
        let machines =
            Self::generate_all_possible_machines(cards, num_cards,
                                                 num_configs);

        Self(machines)
    }

    fn generate_all_possible_cards(inst: Vec<Instructions>,
                                   num_symbols: usize) -> Vec<Card> {
        let mut res: Vec<Vec<Instructions>>    = Vec::new();
        let mut inst_holder: Vec<Instructions> = Vec::new();
        generate_combinations(&inst, num_symbols, 0, 0,
                              &mut inst_holder, &mut res);
        let res: Vec<Card> = res.iter()
                                .map(|x| Card::new(x.clone()))
                                .collect();
        res
    }

    fn generate_all_possible_machines(cards: Vec<Card>,
                                      num_cards: usize,
                                      num_configs: usize) -> Vec<Cards> {
        let mut res: Vec<Vec<Card>>    = Vec::with_capacity(num_configs);
        let mut card_holder: Vec<Card> = Vec::new();
        println!("memory allocated, proceeding...");
        generate_combinations(&cards, num_cards, 0, 0,
                              &mut card_holder, &mut res);
        let res: Vec<Cards> =
            res.into_iter().map(|x| Cards::new(x)).collect();
        res.into_iter() .filter(|x| x.contains_halt_state()).collect()
    }


    fn generate_instructions(num_cards: usize,
                             num_symbols: usize) -> Vec<Instructions> {
        let mut sym = vec![];
        for i in 0..num_symbols {
            sym.push(Symbol(i as u8));
        }

        let dir = [Direction::Left, Direction::Right];
        let mut states = vec![State::Halt];
        for i in 0..num_cards {
            states.push(State::Card(i));
        } 

        let mut res = Vec::new();
        for s in states.iter() {
            for d in dir.iter() {
                for sy in sym.iter() {
                    res.push(Instructions::new(*sy, *d, *s));
                }
            }
        }
        res
    }
}

impl Stats {
    pub fn init(cards: Cards) -> Self {
        Self { cards, action: 0, score: 0,
               boards: vec![], head_positions: vec![] }
    }
    
    pub fn default() -> Self {
        let inst = Instructions::new(Symbol(0), Direction::Left, State::Halt);
        let card = Card::new(vec![inst]);
        Self { cards: Cards::new(vec![card]), action: 0, score: 0,
               boards: vec![], head_positions: vec![] }
    }

    pub fn update(&mut self, board: &Board, tw: &Typewriter) {
        self.score = board.0.iter().filter(|x| **x == 1)
                            .collect::<Vec<&u8>>().len();
        self.boards.push(board.clone());
        self.action = self.boards.len()-1;
        self.head_positions.push(tw.head);
    }

    pub fn show_state(&self) {
        let mut terminal = term::stdout().unwrap();
        for (ind, board) in self.boards.iter().enumerate() {
            for (c,x) in board.0.iter().enumerate() {
                if c == self.head_positions[ind] {
                    terminal.fg(term::color::RED).unwrap();
                    print!("{} ", x);
                    terminal.reset().unwrap();
                } else {
                    if *x as usize == 1 {
                        terminal.fg(term::color::BRIGHT_BLUE).unwrap();
                        terminal.attr(term::Attr::Bold).unwrap();
                        print!("{} ", x);
                        terminal.reset().unwrap();
                    } else {
                        print!("{} ", x);
                    }
                }
            }
            println!("");
        }
        println!("");
    }
}

fn generate_combinations<T: Clone>(input: &Vec<T>,
                              max_depth: usize,
                              mut ind: usize,
                              r: usize,
                              mut input_holder: &mut Vec<T>,
                              mut res: &mut Vec<Vec<T>>) {
    while ind < input.len() {
        input_holder.push(input[ind].clone());
        if r < max_depth-1 {
            generate_combinations(
                input, max_depth, 0, r+1, &mut input_holder, &mut res);
        } else {
            res.push(input_holder.clone());
        }
        input_holder.pop();
        ind += 1;
     }
}

fn busy_beaver(cards: &Cards) -> Option<Stats> {
    let mut board        = Board(vec![0; BOARD_LENGTH]);
    let mut tw           = Typewriter { head: BOARD_LENGTH/2 };
    let mut stats        = Stats::init(cards.clone());
    let mut current_card = cards.get_card(0);
    let mut rounds       = 0;
    stats.update(&board, &tw);
    loop {
        let inst  = current_card.get_instruction(&tw, &board);
        let (state, dir, write) = (&inst.state, &inst.direction, &inst.write);
        tw.write(write, &mut board);
        if !tw.move_head(dir) { break }
        stats.update(&board, &tw);
        match state {
            State::Card(i) => current_card = cards.get_card(*i),
            State::Halt    => return Some(stats),
        }
        rounds += 1;
        if rounds > 100 { break }
    }
    None
}

fn highest_score(stats: &Vec<Stats>) -> &Stats {
    let mut current_leader = &stats[0];
    for s in stats.iter() {
        if s.score > current_leader.score {
            current_leader = s;
        }
    }
    current_leader
}

fn highest_action(stats: &Vec<Stats>) -> &Stats {
    let mut current_leader = &stats[0];
    for s in stats.iter() {
        if s.action > current_leader.action {
            current_leader = s;
        }
    }
    current_leader
}

//TODO: comment all code
fn main() {
    let args: Vec<String> = env::args().collect();

    println!("{:?}", args);

    let arg1 = args[args.len()-2].parse().unwrap();
    let arg2 = args[args.len()-1].parse().unwrap();

    let m = Machines::generate(arg1, arg2);

    println!("Machines generated, moving on to the BUSY BEAVER!");
    
    let mut best_score = 0;
    let mut best_score_holder = Stats::default();
    let mut best_action = 0;
    let mut best_action_holder = Stats::default();

    for (x,c) in m.0.iter().enumerate() {
        if let Some(stats) = busy_beaver(&c) {
            if stats.action > best_action {
                best_action_holder = stats.clone();
                best_action = stats.action;
            }
            if stats.score > best_score {
                best_score_holder = stats.clone();
                best_score = stats.score;
            }
            //stats_holder.push(stats);
        }
    }
    
    println!("winner_score  - score: {:?}  -  action: {:?}",
        best_score_holder.score, best_score_holder.action);
    best_score_holder.show_state();
    /*
    let winner_score = highest_score(&stats_holder);
    let winner_action = highest_action(&stats_holder);
 
    println!("winner_action  -  cards: {:?}  -  score: {:?}  -  action: {:?}",
        winner_score.cards, winner_score.score, winner_score.action);
    winner_score.show_state();
    println!("");
    println!("winner_score  -  cards: {:?}  -  score: {:?}  -  action: {:?}",
        winner_action.cards, winner_action.score, winner_action.action);
    winner_action.show_state();
    */

}

