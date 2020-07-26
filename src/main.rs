extern crate term;

const BOARD_LENGTH: usize = 100;

//#[derive(Debug)]
/// Defines the board/tape on which the busy beaver game will unfold
/// The board in theory should be of inifinte length, but that is far
/// longer than what we need - phew. We set it to BOARD_LENGTH istead!
struct Board([u8; BOARD_LENGTH]);

#[derive(Debug, Clone, Copy)]
/// Directions the 'typewriter' (tw) can move.
enum Direction {
    Left,
    Right,
}

#[derive(Debug, Clone, Copy)]
/// The choosen symbols for our tw that can be written on the tape
enum Symbol {
    Zero,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// States of the the Turing Machine
enum State {
    Card(usize),
    Halt,
}

#[derive(Debug)]
/// Defines a card holding instructions.
struct Cards(Vec<Card>);

#[derive(Debug, Clone)]
/// Defines a card holding instructions.
struct Card([Instructions; 2]);

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

/// Holds all the generated machines
struct Machines(Vec<Cards>);

impl Instructions {
    pub fn new(write: Symbol, direction: Direction, state: State) -> Self {
        Self { write, direction, state }
    }
}

impl Card {
    pub fn new(instructions: [Instructions; 2]) -> Self {
        Self(instructions)
    }

    /// fetch the instructions corresponding to the read symbol
    /// on the board.
    pub fn get_instruction<'a>(
        &'a self, tw: &Typewriter, board: &Board) -> &'a Instructions {
        match tw.read(board) {
            Symbol::Zero => &self.0[0],
            Symbol::One  => &self.0[1],
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
            Symbol::Zero => board.0[self.head] = 0,
            Symbol::One  => board.0[self.head] = 1,
        }
    }

    /// Reads symbol of the board of which the tw head is hovering.
    pub fn read(&self, board: &Board) -> Symbol {
        match board.0[self.head] {
            0 => Symbol::Zero,
            1 => Symbol::One,
            _ => panic!("Undefined symbol for this input")
        }
    }
}

impl Machines {
    pub fn generate(num_cards: usize) -> Self {
        let mut machines: Vec<Cards> = Vec::new();

        // construct every possible set of instructions
        let inst1 = Self::generate_instructions(num_cards);
        let inst2 = inst1.clone();

        // constructs all possible cards
        let mut cards = vec![];
        for i1 in inst1.iter() {
            for i2 in inst2.iter() {
                let card = Card::new([*i1,*i2]);
                cards.push(card);
            }
        }

        // constructs all possible machines
        let machines = Self::generate_all_possible_machines(cards, num_cards);

        println!("number of machines: {:?}", machines.len());

        Self(machines)
    }

    fn combine_cards(cards: &Vec<Card>,
                     num_cards: usize,
                     mut ind: usize,
                     r: usize,
                     mut cards_holder: &mut Vec<Card>,
                     mut res: &mut Vec<Cards>) {
        while ind < cards.len() {
            cards_holder.push(cards[ind].clone());
                
            if r < num_cards {
                Self::combine_cards(
                    cards, num_cards, 0, r+1, &mut cards_holder, &mut res);
            } else {
                let card = Cards::new(cards_holder.to_vec());
                if Self::contains_halt_state(&card) {
                    res.push(card);
                }
            }
            cards_holder.pop();
            ind += 1;
        } 
    }

    fn generate_all_possible_machines(cards: Vec<Card>,
                                      num_cards: usize) -> Vec<Cards> {
        let mut res: Vec<Cards> = Vec::new();
        let mut card_holder: Vec<Card> = Vec::new();
        Self::combine_cards(&cards, num_cards-1, 0, 0, &mut card_holder, &mut res);
        res
    }

    fn contains_halt_state(cards: &Cards) -> bool {
        cards.get_states().contains(&State::Halt)
    }

    fn generate_instructions(num_cards: usize) -> Vec<Instructions> {
        let sym = [Symbol::Zero, Symbol::One];
        let dir = [Direction::Left, Direction::Right];
        let mut states = vec![State::Halt];
        for i in 0..num_cards {
            states.push(State::Card(i));
        } 
//
        let mut res = Vec::new();
        for s in states.iter() {
            for d in dir.iter() {
                for sy in sym.iter() {
                    println!("{:?} {:?} {:?}", sy, d, s);
                    res.push(Instructions::new(*sy, *d, *s));
                }
            }
        }
        res
    }
}

fn show_current_state(board: &Board, tw: &Typewriter) {
    let mut terminal = term::stdout().unwrap();
    for (c,x) in board.0.iter().enumerate() {
        if c == tw.head {
            terminal.fg(term::color::RED).unwrap();
            print!("{} ",x);
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
    print!("\n");
}

fn busy_beaver(
    mut board: &mut Board, tw: &mut Typewriter, cards: &Cards) -> usize {
    let mut current_card = cards.get_card(0);

    let mut rounds = 0;
    println!("starting the busy beaver");
    show_current_state(&board, &tw);
    loop {
        let inst  = current_card.get_instruction(&tw, &board);
        let (state, dir, write) = (&inst.state, &inst.direction, &inst.write);
        tw.write(write, &mut board);
        if !tw.move_head(dir) { println!("out of TAPE"); break }
        show_current_state(&board, &tw);
        match state {
            State::Card(i) => current_card = cards.get_card(*i),
            State::Halt    => break,
        }
        rounds += 1;
        if rounds > 100 { break }
    }
    println!("busy beaver finished");
    0
}

fn main() {

    let i1 = Instructions::new(Symbol::One, Direction::Left, State::Card(0));
    //let i1 = Instructions::new(Symbol::One, Direction::Right, State::Halt);
    let i2 = Instructions::new(Symbol::Zero, Direction::Right, State::Card(0));
    let card = Card::new([i1, i2]);
    let cards = Cards::new(vec![card]);

    let m = Machines::generate(2);

    for c in m.0.iter() {
        let mut board = Board([0; BOARD_LENGTH]);
        let mut tw = Typewriter { head: BOARD_LENGTH/2 };
        println!("{:?}", c);

        busy_beaver(&mut board, &mut tw, &c);
    }


    //busy_beaver(board, tw, cards);
}

