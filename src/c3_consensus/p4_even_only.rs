//! In the previous chapter, we considered a hypothetical scenario where blocks must contain an even
//! state root in order to be valid. Now we will express that logic here as a higher-order consensus
//! engine. It is higher- order because it will wrap an inner consensus engine, such as PoW or PoA
//! and work in either case.

use std::marker::PhantomData;

use super::p1_pow;
use crate::hash;

use super::{ Consensus, Header};



/// A Consensus engine that wraps another consensus engine. This engine enforces the requirement
/// that a block must have an even state root in order to be valid

/// A Consensus engine that requires the state root to be even for the header to be valid.
/// Wraps an inner consensus engine whose rules will also be enforced.
pub struct EvenOnly<Inner: Consensus>{
	pub(crate)  inner_c : Inner,
}

impl<Inner: Consensus> Consensus for EvenOnly<Inner> {
	type Digest = Inner::Digest;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		let mut valid  = self.inner_c.validate(parent_digest,header);
		valid &= header.state_root % 2 == 0;
		valid
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {

		let h =  Header{
			parent: partial_header.parent,
			height: partial_header.height,
			state_root: if partial_header.state_root % 2 == 0 {partial_header.state_root} else {partial_header.state_root+1},
			extrinsics_root: partial_header.extrinsics_root,
			consensus_digest: (),
		};

		return self.inner_c.seal(parent_digest, h);
	}

	fn create_default_instance() -> Self{
		return Self {
			inner_c: Inner::create_default_instance(),
		};
	}
}

/// Using the moderate difficulty PoW algorithm you created in section 1 of this chapter as the
/// inner engine, create a PoW chain that is valid according to the inner consensus engine, but is
/// not valid according to this engine because the state roots are not all even.
fn almost_valid_but_not_all_even() -> Vec<Header<u64>> {

	let mut chain:Vec<Header<u64>>  = vec![]; 

	let c:EvenOnly::<p1_pow::PoW> = EvenOnly::<p1_pow::PoW> {
		inner_c: p1_pow::PoW::new(u64::max_value()/10)
	};

	let mut genesis = Header {
		parent:0,
		height:0,
		state_root:0,
		extrinsics_root:0,
		consensus_digest:0,
	};

	while hash(&genesis) < c.inner_c.get_threashold() {
		genesis.consensus_digest += 10;
	}

	chain.push(genesis);

	for i in 1..5 {
		let iu64 = i as u64;
		let partial_header = Header::<()> {
			parent:chain[i-1].parent,
			height:chain[i-1].height + 1,
			state_root: iu64,
			extrinsics_root:hash(&vec![1+iu64,2+iu64,3+iu64]),
			consensus_digest:(),
		};
		let mut new_header = c.seal(&chain[i-1].consensus_digest, partial_header);
		

		match new_header {
			Some(mut nh) => {
				nh.state_root = iu64;
				chain.push(nh)
			},
			_ => ()
		}

	}

	return chain;

}


#[test]
fn test_almost_valid_but_not_all_even(){

	let chain = almost_valid_but_not_all_even();
	let mut all_even = true;
	for h in chain.iter(){
		println!("{:?}",h);
		all_even &= h.state_root % 2 == 0;
	}
	assert!(!all_even);
}
