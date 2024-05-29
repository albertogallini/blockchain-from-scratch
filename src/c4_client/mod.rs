/// Brain dump (TODO revise later):
///
/// We have a blockchain data structure featuring:
/// 1. A built in addition accumulator state machine
/// 2. A built-in pow consensus mechanism
///
/// We also have abstractions over:
/// 1. State Machines
/// 2. Consensus Engines
///
/// Let's refactor our blockchain to take advantage of these two abstractions
/// In doing so, we create a blockchain framework
use crate::c1_state_machine::StateMachine;
use crate::c3_consensus::{Consensus, Header};
use crate::hash;
type Hash = u64;
use  num::traits::{Zero,One};

impl<Digest> Header<Digest>  
	where Digest: Zero+One+core::hash::Hash {

	/// Returns a new valid genesis header.
	fn genesis(genesis_state_root: Hash) -> Self {
		Header::<Digest>{
			parent: 0,
			height: 0,
			state_root: genesis_state_root,
			extrinsics_root: 0,
			consensus_digest: Digest::zero(),
		}
	}

	/// Create and return a valid child header.
	fn child(&self, state_root: Hash, extrinsics_root: Hash) -> Self {
		return Header::<Digest>{
			 parent:hash(self),
			 height:self.height + 1,
			 state_root : state_root,
			 extrinsics_root: extrinsics_root,
			 consensus_digest: Digest::one(),
		}
	}

	/// Verify a single child header.
	fn verify_child(&self, child: &Self) -> bool {
		 hash(self) == child.parent
		 &&
		 self.height == child.height - 1
	}

	/// Verify that all the given headers form a valid chain from this header to the tip.
	fn verify_sub_chain(&self, chain: &[Self]) -> bool {
		let chain_len = chain.len();
		if chain_len == 0 {
			return true; // it is just an empty
		}
		if chain_len == 1 {
			return false; // cannot verify
		}
		for block_i in 0..chain_len-1 {
			if chain[block_i].verify_child(&chain[block_i+1]) {
				return false;
			}
		}
		return true;
	}
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct Block<C: Consensus, SM: StateMachine> {
	header: Header<C::Digest>,
	body: Vec<SM::Transition>,
	consensus : C,
}

impl<C: Consensus, SM: StateMachine> Block<C, SM>  
	where 
	SM::State :core::hash::Hash + Clone,
	<C as Consensus>::Digest: Zero+One+core::hash::Hash {

	

	/// Returns a new valid genesis block. By convention this block has no extrinsics.
	pub fn genesis(genesis_state: &SM::State) -> Self {
		 Block::<C,SM>{
			header: Header::<C::Digest>::genesis(hash(genesis_state)),
			body : vec![],
			consensus  : C::create_default_instance(),
		 }
	}

	/// Create and return a valid child block.
	pub fn child(&self, pre_state: &SM::State, extrinsics: Vec<u8>) -> Self {

		let mut s  = pre_state.clone();
		for t in self.body.iter() {
			s = SM::next_state(&s, &t).clone();
		}

		
		let h = Header::<()>{
			parent : hash(&self.header),
			state_root : hash(&s),
			height : self.header.height,
			extrinsics_root : hash(&extrinsics[0]),
			consensus_digest : (),
		};

		let ch = self.consensus.seal(&C::Digest::one(), h);
		match ch {
			Some(ch) => {
				Block::<C,SM>{
					header: ch,
					body : vec![], // TODO : how can I define the block transition !???? 
					consensus:  C::create_default_instance()
				 }
			}
			None => {
				let hh = Header::<C::Digest>{
					parent : hash(&self.header),
					state_root : hash(&s),
					height : self.header.height,
					extrinsics_root : hash(&extrinsics[0]),
					consensus_digest : C::Digest::one(), // we return a defult block with just one digest .. we should retunr None and change the return type of this function to Option .. 
				};
				Block::<C,SM>{
					header: hh,
					body : vec![], // TODO : how can I define the block transition !???? 
					consensus:  C::create_default_instance()
				 }
			}
		}
		

	}

	/// Verify that all the given blocks form a valid chain from this block to the tip.
	pub fn verify_sub_chain(&self, pre_state: &SM::State, chain: &[Self]) -> bool {
		let mut s  = pre_state.clone();
		let mut check = true;
		
		for i  in 1..chain.len() {
			for t in chain[i-1].body.iter() {
				s = SM::next_state(&s, &t).clone();
			} 
			check &= chain[i-1].header.state_root == hash(&s);
			check &= hash(&chain[i-1].header) == chain[i].header.parent;
		}	
		check
	}
}


/// Create and return a block chain that is n blocks long starting from the given genesis state.
/// The blocks should not contain any transactions.
fn create_empty_chain<C: Consensus, SM: StateMachine>(
	n: u64,
	genesis_state: &SM::State,
) -> Vec<Block<C, SM>> 
where 
SM::State : core::hash::Hash + Clone,
<C as Consensus>::Digest: Zero+One+core::hash::Hash {

	let mut chain:Vec<Block<C, SM>> = vec![];
	let mut b = Block::<C, SM>::genesis(genesis_state);
	let mut pre_state = genesis_state.clone();
	chain.push(b);
	for i in 1..n as usize {
		
		let tb = chain[i-1].child(&pre_state, vec![]);

		chain.push(tb);
		let mut s  = pre_state.clone();
		for t in chain[i].body.iter() {
			s = SM::next_state(&s, &t).clone();
		}
		pre_state = s;
		
	}
	chain
}

//TODO tests

//TODO maybe this shouldn't be a whole chapter. Maybe it is the first
// section in the chapter on building a client
