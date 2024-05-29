//! We saw in the previous chapter that blockchain communities sometimes opt to modify the
//! consensus rules from time to time. This process is knows as a fork. Here we implement
//! a higher-order consensus engine that allows such forks to be made.
//!
//! The consensus engine we implement here does not contain the specific consensus rules to
//! be enforced before or after the fork, but rather delegates to existing consensus engines
//! for that. Here we simply write the logic for detecting whether we are before or after the fork.

use std::{any::TypeId, marker::PhantomData};

use super::{p4_even_only::EvenOnly, p1_pow::PoW, p3_poa::SimplePoa, Consensus, ConsensusAuthority, Header};

/// A Higher-order consensus engine that represents a change from one set of consensus rules
/// (Before) to another set (After) at a specific block height
struct Forked<D, Before, After> {
	/// The first block height at which the new consensus rules apply
	fork_height: u64,
	phdata: PhantomData<D>,
	inner_c_after  : After,
	inner_c_before : Before,
}


impl<D, B, A> Consensus for Forked<D, B, A>
where
	D: Clone + core::fmt::Debug + Eq + PartialEq + std::hash::Hash,
	B: Consensus,
	A: Consensus,
	B::Digest: Into<D>,
	A::Digest: Into<D>,
	<A as Consensus>::Digest: From<D>,
	<B as Consensus>::Digest: From<D>,

{
	type Digest = D;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) 
			-> bool {
		if header.height > self.fork_height {
			let digest_after:A::Digest = <D as Into<A::Digest>>::into(parent_digest.clone());
			let header_after = Header::<A::Digest>{
				parent: header.parent,
				height: header.height,
				state_root: header.state_root,
				extrinsics_root: header.extrinsics_root,
				consensus_digest: <D as Into<A::Digest>>::into(header.consensus_digest.clone()),
			};
			return self.inner_c_after.validate(&digest_after, &header_after);
		} 
		else {
			let digest_before:B::Digest = <D as Into<B::Digest>>::into(parent_digest.clone());
			let header_before = Header::<B::Digest>{
				parent: header.parent,
				height: header.height,
				state_root: header.state_root,
				extrinsics_root: header.extrinsics_root,
				consensus_digest: <D as Into<B::Digest>>::into(header.consensus_digest.clone()),
			};
			return self.inner_c_before.validate(&digest_before, &header_before);
		}
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
		if partial_header.height > self.fork_height {
			let digest_after:A::Digest = <D as Into<A::Digest>>::into(parent_digest.clone());
			let rheader = self.inner_c_after.seal(&digest_after, partial_header);
			match rheader {
				Some(valueh) => {
					let h = Header::<Self::Digest> {
						parent: valueh.parent,
						height: valueh.height,
						state_root: valueh.state_root,
						extrinsics_root: valueh.extrinsics_root,
						consensus_digest: <<A as Consensus>::Digest as Into<Self::Digest>>::into(valueh.consensus_digest.clone()),
					};
					return Some(h);
				}
				None => {
					None
				}
			}
		} 
		else {
			let digest_before:B::Digest = <D as Into<B::Digest>>::into(parent_digest.clone());
			let rheader = self.inner_c_before.seal(&digest_before, partial_header);
			match rheader {
				Some(valueh) => {
					let h = Header::<Self::Digest> {
						parent: valueh.parent,
						height: valueh.height,
						state_root: valueh.state_root,
						extrinsics_root: valueh.extrinsics_root,
						consensus_digest: <<B as Consensus>::Digest as Into<Self::Digest>>::into(valueh.consensus_digest.clone()),
					};
					return Some(h);
				}
				None => {
					None
				}
			}
		}
	}

	fn create_default_instance() -> Self {
		Self { 
				fork_height:10, 
				phdata: PhantomData::<D>{},
			 	inner_c_after: A::create_default_instance(),
			  	inner_c_before: B::create_default_instance() 
			}
	}

fn verify_sub_chain(
			&self,
			parent_digest: &Self::Digest,
			chain: &[Header<Self::Digest>],
		) -> bool {
			let chain_len = chain.len();

			if chain_len == 0 {
				return true; // it is just an empty
			}
			if chain_len == 1 {
				return false; // cannot verify
			}
			else if  chain_len > 2{
				return self.validate(parent_digest,&chain[1]) 
					   && 
					   self.verify_sub_chain(&chain[1].consensus_digest,&chain[1..]);
			}
			else{ // chain_len == 2
				return self.validate(parent_digest,&chain[1]);
			}
		}

fn human_name() -> String {
			"Unnamed Consensus Engine".into()
		}

}

/// Create a PoA consensus engine that changes authorities part way through the chain's history.
/// Given the initial authorities, the authorities after the fork, and the height at which the fork
/// occurs.
fn change_authorities(
	fork_height: u64,
	initial_authorities: Vec<ConsensusAuthority>,
	final_authorities: Vec<ConsensusAuthority>,
) -> impl Consensus {

	Forked::<ConsensusAuthority,SimplePoa,SimplePoa>{
		fork_height : 10,
		inner_c_after : SimplePoa{ authorities:final_authorities},
		inner_c_before: SimplePoa{ authorities:initial_authorities},
		phdata: PhantomData::<ConsensusAuthority>{},

	}

}

/// Create a PoW consensus engine that changes the difficulty part way through the chain's history.
fn change_difficulty(
	fork_height: u64,
	initial_difficulty: u64,
	final_difficulty: u64,
) -> impl Consensus {
	Forked::<u64,PoW,PoW>{
		fork_height : 10,
		inner_c_after : PoW{ threshold:final_difficulty},
		inner_c_before: PoW{ threshold:initial_difficulty},
		phdata: PhantomData::<u64>{},

	}
}

/// Earlier in this chapter we implemented a consensus rule in which blocks are only considered
/// valid if they contain an even state root. Sometimes a chain will be launched with a more
/// traditional consensus like PoW or PoA and only introduce an additional requirement like even
/// state root after a particular height.
///
/// Create a consensus engine that introduces the even-only logic only after the given fork height.
/// Other than the evenness requirement, the consensus rules should not change at the fork. This
/// function should work with either PoW, PoA, or anything else as the underlying consensus engine.
fn even_after_given_height<Original: Consensus>(fork_height: u64) -> impl Consensus {

	return Forked::< Original::Digest, Original,EvenOnly::<Original> > {
		fork_height : fork_height,
		inner_c_after : EvenOnly::<Original>::create_default_instance(),
		inner_c_before: Original::create_default_instance(),
		phdata: PhantomData::<<Original>::Digest>{},
	};
	
}

/// So far we have considered the simpler case where the consensus engines before and after the fork
/// use the same Digest type. Let us now turn our attention to the more general case where even the
/// digest type changes.
///
/// In order to implement a consensus change where even the Digest type changes, we will need an
/// enum that wraps the two individual digest types
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
enum PowOrPoaDigest {
	Pow(u64),
	Poa(ConsensusAuthority),
}

impl From<u64> for PowOrPoaDigest {
	fn from(d: u64) -> Self {
		PowOrPoaDigest::Pow(d)
	}
}

impl From<ConsensusAuthority> for PowOrPoaDigest {
	fn from(d: ConsensusAuthority) -> Self {
		PowOrPoaDigest::Poa(d)
	}
}

impl From<PowOrPoaDigest> for ConsensusAuthority {
    fn from(d: PowOrPoaDigest) -> Self {
        match d {
            PowOrPoaDigest::Pow(_) => ConsensusAuthority::Alice,
            PowOrPoaDigest::Poa(authority) => authority,
        }
    }
}

impl From<PowOrPoaDigest> for u64 {
    fn from(d: PowOrPoaDigest) -> Self {
        match d {
            PowOrPoaDigest::Pow(value) => value,
            PowOrPoaDigest::Poa(_) =>  10
        }
    }
}

/// In the spirit of Ethereum's recent switch from PoW to PoA, let us model a similar
/// switch in our consensus framework. It should go without saying that the real-world ethereum
/// handoff was considerably more complex than it may appear in our simplified example, although
/// the fundamentals are the same.
/// 
type ForkedPoaPow = Forked::< PowOrPoaDigest, PoW, SimplePoa >;

fn pow_to_poa(
	fork_height: u64,
	difficulty: u64,
	authorities: Vec<ConsensusAuthority>,
) -> ForkedPoaPow {

	return ForkedPoaPow {
		fork_height : fork_height,
		inner_c_before: PoW{ 
			threshold: difficulty 
		},
		inner_c_after: SimplePoa{
			authorities:authorities
		},
		phdata: PhantomData::<PowOrPoaDigest>{},
	}
}




use crate::hash;
#[test]
fn test_consensus_pow_to_poa(){
	
	const CHAIN_LEN:usize = 50;
	let consensus = pow_to_poa(
		 10,
		 u64::max_value()/100,
		 vec![ConsensusAuthority::Bob, ConsensusAuthority::Charlie]);


	let genesis: Header<PowOrPoaDigest> = Header {
		parent:0,
		height:0,
		state_root:0,
		extrinsics_root:0,
		consensus_digest:PowOrPoaDigest::Pow(0)
	};

	//let chain:Vec< Header<<Forked::< PowOrPoaDigest, PoW, SimplePoa > as Consensus>::Digest >>  = vec![];
	let mut chain:Vec< Header<PowOrPoaDigest> > = vec![];
	chain.push(genesis.into());

	for i in 1..CHAIN_LEN {
		let iu64 = i as u64;
		let  partial_header  = Header {
			parent:hash(&chain[i-1]),
			height:iu64,
			state_root:iu64 * 10,
			extrinsics_root:hash(&vec![1+iu64,2+iu64,3+iu64]),
			consensus_digest:(),
		};
		let parent_digest: &PowOrPoaDigest = &chain[i-1].consensus_digest;
		let header = consensus.seal(parent_digest, partial_header);
		match header {
			Some(header) => {
				chain.push(header);
			}
			None => {
				break;
			}
		}
	}

	for i in 0..CHAIN_LEN {
		if chain[i].height <= consensus.fork_height  {
			match chain[i].consensus_digest {
				PowOrPoaDigest::Pow(_v) => {assert!(true)}
				_ => {assert!(false)}
			}
		}
		else {
			match chain[i].consensus_digest {
				PowOrPoaDigest::Poa(_v) => {assert!(true)}
				_ => {assert!(false)}
			}
		}
	}

	

}
