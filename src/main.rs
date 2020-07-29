extern crate term;

use::std::env;

#[derive(Debug, Clone)]
/// Defines the tape/tape on which the busy beaver game will unfold
/// The tape in theory can be of inifinte length. In the real world
/// we are limited by RAM.
struct Tape(Vec<u8>);

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
/// Holds the instructions of the different states.
struct Card(Vec<Instructions>);

#[derive(Debug, Clone, Copy)]
/// Holds the instruction the tw must follow
struct Instructions {
    /// Symbol to be written on the tape
    write: Symbol,

    /// The direction the tw moves after write
    direction: Direction,

    /// State of next operation
    state: State,
}

/// Typewriter holds and controls the position of the moving head 
struct Typewriter {
    head: usize,
}

#[derive(Debug)]
/// Holds all the generated machines
struct Machines(Vec<Cards>);

#[derive(Debug, Clone)]
/// Keep track of stats during a run of the Busy Beaver
struct Stats {
    cards:          Cards,
    score:          usize,
    action:         usize,
    tapes:          Vec<Tape>,
    head_positions: Vec<i64>,
}

impl Tape {
    /// extends the tape by putting a zero at the beginning and end of the
    /// current tape
    pub fn extend(&mut self, tw: &mut Typewriter) {
        tw.head = tw.head + 1;
        self.0.insert(0,0);
        self.0.push(0);
    }
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
    /// on the tape.
    pub fn get_instruction<'a>(&'a self, tw: &Typewriter,
                               tape: &Tape) -> &'a Instructions {
        match tw.read(tape) {
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
    pub fn move_head(&mut self, dir: &Direction, tape: &mut Tape) {
        match dir {
            Direction::Left  => self.head = self.head - 1,
            Direction::Right => self.head = self.head + 1,
        }

        if self.head < 0 || self.head > (tape.0.len()-1) {
            tape.extend(self);
        }
    }

    /// Writes the new symbol onto the tape.
    pub fn write(&self, sym: &Symbol, tape: &mut Tape) {
        match sym {
            Symbol(i) => tape.0[self.head] = *i,
        }
    }

    /// Reads symbol of the tape of which the tw head is hovering.
    pub fn read(&self, tape: &Tape) -> Symbol {
        Symbol(tape.0[self.head])
    }
}

impl Machines {
    /// Generates all possible machines that contains the halt-state
    pub fn generate(num_cards: usize, num_symbols: usize) -> Self {
        // construct every possible set of instructions
        let inst = Self::generate_instructions(num_cards, num_symbols);

        // constructs all possible cards
        let cards = Self::generate_all_possible_cards(inst, num_symbols);

        // calculates the number of machines created. NUmber is used to pre-allocate
        // the needed memory to store them all.
        let num_allocate = 
            (num_symbols*2*(num_cards+1)).pow((num_cards+num_symbols) as u32);

        // constructs all possible machines that contains the halt state
        println!("Starts creating machines");
        let machines =
            Self::generate_all_possible_machines(cards, num_cards,
                                                 num_allocate);

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
                                      num_allocate: usize) -> Vec<Cards> {
        let mut res: Vec<Vec<Card>>    = Vec::with_capacity(num_allocate);
        let mut card_holder: Vec<Card> = Vec::new();
        println!("memory allocated, proceeding...");
        generate_combinations(&cards, num_cards, 0, 0,
                              &mut card_holder, &mut res);
        let res: Vec<Cards> = res.into_iter()
                                 .map(|x| Cards::new(x))
                                 .collect();
        res.into_iter().filter(|x| x.contains_halt_state()).collect()
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
               tapes: vec![], head_positions: vec![] }
    }
    
    pub fn default() -> Self {
        let inst = Instructions::new(Symbol(0), Direction::Left, State::Halt);
        let card = Card::new(vec![inst]);
        Self { cards: Cards::new(vec![card]), action: 0, score: 0,
               tapes: vec![], head_positions: vec![] }
    }

    /// update all stats based on state of the tape and typewriter
    pub fn update(&mut self, tape: &Tape, tw: &Typewriter) {
        self.score = tape.0.iter().filter(|x| **x == 1)
                            .collect::<Vec<&u8>>().len();
        self.tapes.push(tape.clone());
        self.action = self.tapes.len()-1;
        self.head_positions.push((tw.head-tape.0.len()/2) as i64);
    }

    /// shows the all the states the busy beaver went through.
    /// All ones are printed in blue color, while the current position
    /// of the head is printed in red.
    pub fn show_state(&self) {
        let mut terminal = term::stdout().unwrap();

        let tape_length = self.tapes[self.tapes.len()-1].0.len();
        let mid = tape_length/2;

        // Make all taped the same length as the final tape by padding zeroes.
        // Proceed to insert values at positions relative to the center of
        // the tape..
        let mut formatted_tapes = vec![];
        let mut head_positions = self.head_positions.clone() as Vec<i64>;
        for (ind, tape) in self.tapes.iter().enumerate() {
            let mut tmp = vec![0u8; tape_length];
            let start = mid-(tape.0.len()/2);
            head_positions[ind] = (mid as i64)+head_positions[ind];
            for (c,t) in tape.0.iter().enumerate() {
                tmp[start+c] = *t;
            }
            formatted_tapes.push(tmp);
        }

        // prints the tapes in order and with color
        for (ind, tape) in formatted_tapes.iter().enumerate() {
            for (c,x) in tape.iter().enumerate() {
                if c == (head_positions[ind] as usize) {
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

/// generates all possible combinations of the elements in the input vec with 
/// a length of max_depth
/// # Example
/// input -> vec [1,2], max_depth -> 2
/// gives res -> [1,1], [1,2], [2,1]. [2,2] 
fn generate_combinations<T: Clone>(input: &Vec<T>, max_depth: usize,
                                   mut ind: usize, r: usize,
                                   mut input_holder: &mut Vec<T>,
                                   mut res: &mut Vec<Vec<T>>) {
    while ind < input.len() {
        input_holder.push(input[ind].clone());

        // calls itself recursively until the max-depth is reached
        if r < max_depth-1 {
            generate_combinations(input, max_depth, 0, r+1,
                                  &mut input_holder, &mut res);
        } else {
            res.push(input_holder.clone());
        }
        input_holder.pop();
        ind += 1;
    }
}

/// Runs the busy beavers, you feed it the card with instructions to follow
/// and this functions runs them. Busy beaver will break if we're out of tape
/// or if (at least for now (might be changed later if feel for it))
/// we go over 100 iterations/rounds. If it happens that we run into the halt-
/// state, then we return the stats.
fn busy_beaver(cards: &Cards) -> Option<Stats> {
    let mut tape         = Tape(vec![0]);
    let mut tw           = Typewriter { head: 0 };
    let mut stats        = Stats::init(cards.clone());
    let mut current_card = cards.get_card(0);
    let mut rounds       = 0;
    stats.update(&tape, &tw);
    loop {
        let inst = current_card.get_instruction(&tw, &tape);
        let (state, dir, write) = (&inst.state, &inst.direction, &inst.write);
        tw.write(write, &mut tape);

        // Moves the head if possible. If we're out of space -> break
        tw.move_head(dir, &mut tape);
        stats.update(&tape, &tw);

        // Change state
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

    let machines = Machines::generate(arg1, arg2).0;

    println!("Machines generated, moving on to the BUSY BEAVER!");
    
    let mut best_score = 0;
    let mut best_score_holder = Stats::default();
    let mut best_action = 0;
    let mut best_action_holder = Stats::default();

    for m in machines.iter() {
        if let Some(stats) = busy_beaver(&m) {
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
    
    println!("winner_score  - score: {:?}  -  action: {:?}\n cards - {:?}",
        best_score_holder.score, best_score_holder.action,
        best_score_holder.cards);
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

