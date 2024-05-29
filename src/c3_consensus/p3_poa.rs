//! Proof of Work is very energy intensive but is decentralized. Dictator is energy cheap, but
//! is completely centralized. Let's achieve a middle ground by choosing a set of authorities
//! who can sign blocks as opposed to a single dictator. This arrangement is typically known as
//! Proof of Authority.
//!
//! In public blockchains, Proof of Authority is often moved even further toward the decentralized
//! and permissionless end of the spectrum by electing the authorities on-chain through an economic
//! game in which users stake tokens. In such a configuration it is often known as "Proof of Stake".
//! Even when using the Proof of Stake configuration, the underlying consensus logic is identical to
//! the proof of authority we are writing here.

use super::{Consensus, ConsensusAuthority, Header};

/// A Proof of Authority consensus engine. If any of the authorities have signed the block, it is
/// valid.
pub struct SimplePoa {
	pub authorities: Vec<ConsensusAuthority>,
}



impl Consensus for SimplePoa {
	type Digest = ConsensusAuthority;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		(header.consensus_digest == ConsensusAuthority::Charlie 
		||
		header.consensus_digest == ConsensusAuthority::Bob 
		||
		header.consensus_digest == ConsensusAuthority::Alice )

		&&

		(*parent_digest == ConsensusAuthority::Charlie 
		||
		*parent_digest == ConsensusAuthority::Bob 
		||
		*parent_digest == ConsensusAuthority::Alice )

		
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
	
		let author = self.authorities.last();
		
		match author {
			Some(mut a) =>{
				if a == parent_digest {
					a = &self.authorities[0];
				}
				let h= Header::<Self::Digest> {
					parent: partial_header.parent,
					height: partial_header.height,
					state_root: partial_header.state_root,
					extrinsics_root: partial_header.extrinsics_root,
					consensus_digest: *a,
				};
				return Some(h)
			}
			_ => {
				return None;
			}

		}
		
	}
	
	fn create_default_instance() -> Self {
			SimplePoa{
				authorities : vec![ConsensusAuthority::Alice,ConsensusAuthority::Bob,ConsensusAuthority::Charlie],
			}
	}
}

/// A Proof of Authority consensus engine. Only one authority is valid at each block height.
/// As ever, the genesis block does not require a seal. After that the authorities take turns
/// in order.
struct PoaRoundRobinByHeight {
	authorities: Vec<ConsensusAuthority>,
}


impl From<SimplePoa> for PoaRoundRobinByHeight {
	fn from(d: SimplePoa) -> Self {
		PoaRoundRobinByHeight{
			authorities: d.authorities
		}
	}
}

impl Consensus for PoaRoundRobinByHeight {
	type Digest = ConsensusAuthority;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		 if header.height % 3 == 1 {
			return header.consensus_digest == ConsensusAuthority::Bob && *parent_digest == ConsensusAuthority::Alice
		 }
		 else if header.height % 3 == 0 {
			return header.consensus_digest == ConsensusAuthority::Alice && *parent_digest == ConsensusAuthority::Charlie
		 }
		 else {
			 return  header.consensus_digest == ConsensusAuthority::Charlie && *parent_digest == ConsensusAuthority::Bob
		 }
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
		
		let a = match partial_header.height % 3 {
			0 => ConsensusAuthority::Alice,
			1 => ConsensusAuthority::Bob,
			2 => ConsensusAuthority::Charlie,
			_ => {
				return None;
			}
		};

		let parent_is_correct = match a {
			ConsensusAuthority::Alice => { *parent_digest == ConsensusAuthority::Charlie},
			ConsensusAuthority::Bob   => { *parent_digest == ConsensusAuthority::Alice},
			ConsensusAuthority::Charlie => { *parent_digest == ConsensusAuthority::Bob},
		};

		match parent_is_correct {
			true => {
				let h= Header::<Self::Digest> {
					parent: partial_header.parent,
					height: partial_header.height,
					state_root: partial_header.state_root,
					extrinsics_root: partial_header.extrinsics_root,
					consensus_digest: a,
				};
			    Some(h)
			},
			_ => {
				None
			}

		}
	}
	
	fn create_default_instance() -> Self {
		SimplePoa::create_default_instance().into()
	}

}

/// Both of the previous PoA schemes have the weakness that a single dishonest authority can corrupt
/// the chain.
/// - When allowing any authority to sign, the single corrupt authority can sign blocks with invalid
///   transitions with no way to throttle them.
/// - When using the round robin by height, their is throttling, but the dishonest authority can
///   stop block production entirely by refusing to ever sign a block at their height.
///
/// A common PoA scheme that works around these weaknesses is to divide time into slots, and then do
/// a round robin by slot instead of by height
struct PoaRoundRobinBySlot {
	authorities: Vec<ConsensusAuthority>,
}

/// A digest used for PoaRoundRobinBySlot. The digest contains the slot number as well as the
/// signature. In addition to checking that the right signer has signed for the slot, you must check
/// that the slot is always strictly increasing. But remember that slots may be skipped.
#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
struct SlotDigest {
	slot: u64,
	signature: ConsensusAuthority,
}

impl Consensus for PoaRoundRobinBySlot {
	type Digest = SlotDigest;

	fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		if header.consensus_digest.slot % 3 == 1 {
			return header.consensus_digest.signature == ConsensusAuthority::Bob && parent_digest.slot < header.consensus_digest.slot
		 }
		 else if header.height % 3 == 0 {
			return header.consensus_digest.signature == ConsensusAuthority::Alice && parent_digest.slot < header.consensus_digest.slot
		 }
		 else {
			 return  header.consensus_digest.signature == ConsensusAuthority::Charlie && parent_digest.slot < header.consensus_digest.slot
		 }
	}

	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {
		let a = match (parent_digest.slot+1) % 3 {
			0 => ConsensusAuthority::Alice,
			1 => ConsensusAuthority::Bob,
			2 => ConsensusAuthority::Charlie,
			_ => {
				return None;
			}
		};      

		let h= Header::<Self::Digest> {
			parent: partial_header.parent,
			height: partial_header.height,
			state_root: partial_header.state_root,
			extrinsics_root: partial_header.extrinsics_root,
			consensus_digest: SlotDigest{ 
						slot:parent_digest.slot+1, 
						signature:a 
					}
			};

		Some(h)
			
	}

	fn create_default_instance() -> Self{
		return Self {
			authorities: vec![ConsensusAuthority::Alice,ConsensusAuthority::Bob,ConsensusAuthority::Charlie]
		}
	}
}
