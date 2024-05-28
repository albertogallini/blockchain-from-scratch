//! Now is your chance to get creative. Choose a state machine that interests you and model it here.
//! Get as fancy as you like. The only constraint is that it should be simple enough that you can
//! realistically model it in an hour or two.
//!
//! Here are some ideas:
//! * Board games:
//!   * Chess
//!   * Checkers
//!   * Tic tac toe
//! * Beaurocracies:
//!   * Beauro of Motor Vehicles - maintains driving licenses and vehicle registrations.
//!   * Public Utility Provider - Customers open accounts, consume the utility, pay their bill periodically, maybe utility prices fluctuate
//!   * Land ownership registry
//! * Tokenomics:
//!   * Token Curated Registry
//!   * Prediction Market
//!   * There's a game where there's a prize to be split among players and the prize grows over time. Any player can stop it at any point and take most of the prize for themselves.
//! * Social Systems:
//!   * Social Graph
//!   * Web of Trust
//!   * Reputation System

use super::StateMachine;

pub struct TicTacToeSystem;


#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub enum TTTSymbol {
	X,
	O,
	Blank,

}


#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct Board<T, const ROWS: usize, const COLS: usize>{
	data:[[T; COLS]; ROWS],
}

impl<T, const ROWS: usize, const COLS: usize> Board<T, ROWS, COLS> {
    pub fn new(data: [[T; COLS]; ROWS]) -> Self {
        Self { data }
    }
}

const N_ROWS:usize  = 3;
const N_COLS:usize  = 3;
type TTTBoard = Board<TTTSymbol,N_ROWS,N_COLS>;


#[derive(Eq, PartialEq, Debug, Clone, Copy)]
pub struct State {
	board: TTTBoard,
	num_transitions: u8,  // from 0 to 8
	last_mover     : TTTSymbol,
	match_complete:  bool,
}


impl  State {

	pub fn new() -> Self{
		let newd = State{ 
								board:TTTBoard { data:[
									[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
									[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
									[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
									]
								},
								last_mover: TTTSymbol::Blank,
								num_transitions:0,
								match_complete:false,
							} ;
		return newd;
	}

	pub fn reset(&mut self) {
		for r in self.board.data.iter_mut() {
			for c in r {
				*c = TTTSymbol::Blank;
			}
		}
		self.num_transitions = 0;
		self.match_complete = false;
		self.last_mover     = TTTSymbol::Blank;
	}
	
}

pub enum Transition {
	MarkCell{symbol:TTTSymbol, row:usize, col:usize},  
	Reset
}

use std::cell::RefCell;
use std::rc::Rc;

impl StateMachine for TicTacToeSystem {
	type State = Rc<RefCell<State>>;
	type Transition = Transition;

	fn next_state(starting: &Self::State, t: &Self::Transition) -> Self::State {
		
		if starting.borrow().match_complete {
			let new_state = Rc::clone(starting);
			return new_state;
		}

		let new_state = Rc::clone(starting);
		match t {

			Transition::MarkCell{symbol, row, col } => {
				
				if *symbol == starting.borrow().last_mover {
					return  new_state;
				}
				
				if *row < N_ROWS && *col < N_COLS {
					let cs = new_state.borrow().board.data[*row][*col];
					if cs == TTTSymbol::Blank && *symbol != TTTSymbol::Blank {
						starting.borrow_mut().board.data[*row][*col] = *symbol;
						starting.borrow_mut().num_transitions += 1;
						starting.borrow_mut().last_mover = *symbol;
					}

					// horizontal wins
					if 	!starting.borrow_mut().match_complete {
						for r in starting.borrow_mut().board.data {
							if r.iter().all(|&x|  x == TTTSymbol::X) 
							|| 
							r.iter().all(|&x|  x == TTTSymbol::O) {
								starting.borrow_mut().match_complete = true;
							}
						}
					}

					// vertical wins
					if 	!starting.borrow_mut().match_complete {
						for r_index in 0..N_ROWS {
							if  starting.borrow_mut().board.data.iter().all( |&x|  x[r_index] == TTTSymbol::X )  
								|| starting.borrow_mut().board.data.iter().all( |&x|  x[r_index] == TTTSymbol::O ) {
									starting.borrow_mut().match_complete = true;
							} 
						}
					}

					// diagonal wins
					if 	!starting.borrow_mut().match_complete {
						let values  = [TTTSymbol::X,TTTSymbol::O];
						for sym in values.iter() {
							if     starting.borrow_mut().board.data[0][0] == *sym
								&& starting.borrow_mut().board.data[1][1] == *sym
								&& starting.borrow_mut().board.data[2][2] == *sym {
									starting.borrow_mut().match_complete = true;
								}

							else if starting.borrow_mut().board.data[0][2] == *sym 
								 && starting.borrow_mut().board.data[1][1] == *sym
								 && starting.borrow_mut().board.data[2][0] == *sym {
									starting.borrow_mut().match_complete = true;
								}
						}
					}
					
					//no winners
					if !starting.borrow_mut().match_complete && starting.borrow_mut().num_transitions == (N_COLS * N_ROWS) as u8 {
						starting.borrow_mut().match_complete = true;
					}
					
				}
			},

			Transition::Reset => {
				starting.borrow_mut().reset()
			}


		}
		
		return new_state;

	}
}



#[test]
fn reset_new_dashboard() {
	let start = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State::new()));
	let end   = TicTacToeSystem::next_state(&start,&Transition::Reset);
	assert_eq!(end, start);
}

#[test]
fn test_xxin00() {
	let start = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State::new()));
	let end   = TicTacToeSystem::next_state(&start,&Transition::MarkCell { symbol: TTTSymbol::X, row: 0, col: 0 });
	let expected = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
			board:TTTBoard { data:[
				[TTTSymbol::X,TTTSymbol::Blank,TTTSymbol::Blank],
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
				]
			},
			num_transitions:1,
			last_mover:TTTSymbol::X,
			match_complete:false,
	}));
	assert_eq!(end, expected);
}

#[test]
fn test_insertBlankFails() {
	let start = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State::new()));
	let end   = TicTacToeSystem::next_state(&start,&Transition::MarkCell { symbol: TTTSymbol::Blank, row: 0, col: 0 });
	let expected = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
			board:TTTBoard { data:[
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
				]
			},
			num_transitions:0,
			last_mover:TTTSymbol::Blank,
			match_complete:false,
	}));
	assert_eq!(end, expected);
}


#[test]
fn test_diagonal_wis() {
	let start = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
		board:TTTBoard { data:[
			[TTTSymbol::X,TTTSymbol::O,TTTSymbol::Blank],
			[TTTSymbol::O,TTTSymbol::X,TTTSymbol::Blank],
			[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::Blank],
			]
		},
		num_transitions:2,
		last_mover:TTTSymbol::O,
		match_complete:false,
	}));
	let end   = TicTacToeSystem::next_state(&start,&Transition::MarkCell { symbol: TTTSymbol::X, row: 2, col: 2 });

	let expected = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
			board:TTTBoard { data:[
				[TTTSymbol::X,TTTSymbol::O,TTTSymbol::Blank],
				[TTTSymbol::O,TTTSymbol::X,TTTSymbol::Blank],
				[TTTSymbol::Blank,TTTSymbol::Blank,TTTSymbol::X],
				]
			},
			num_transitions:3,
			last_mover:TTTSymbol::X,
			match_complete:true,
	}));
	assert_eq!(end, expected);
}

#[test]
fn test_matchCompleteNoWinner() {
	let start = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
		board:TTTBoard { data:[
			[TTTSymbol::X,TTTSymbol::O,TTTSymbol::O],
			[TTTSymbol::O,TTTSymbol::X,TTTSymbol::X],
			[TTTSymbol::X,TTTSymbol::O,TTTSymbol::Blank],
			]
		},
		num_transitions:8,
		last_mover:TTTSymbol::X,
		match_complete:false,
	}));
	let end   = TicTacToeSystem::next_state(&start,&Transition::MarkCell { symbol: TTTSymbol::O, row: 2, col: 2 });

	let expected = <TicTacToeSystem as StateMachine>::State::new(RefCell::new(State{ 
			board:TTTBoard { data:[
				[TTTSymbol::X,TTTSymbol::O,TTTSymbol::O],
				[TTTSymbol::O,TTTSymbol::X,TTTSymbol::X],
				[TTTSymbol::X,TTTSymbol::O,TTTSymbol::O],
				]
			},
			num_transitions:9,
			last_mover:TTTSymbol::O,
			match_complete:true,
	}));
	assert_eq!(end, expected);
}