//! PoW and PoA each have their own set of strengths and weaknesses. Many chains are happy to choose
//! one of them. But other chains would like consensus properties that fall in between. To achieve
//! this we could consider interleaving PoW blocks with PoA blocks. Some very early designs of
//! Ethereum considered this approach as a way to transition away from PoW.

use crate::hash;

use super::{p1_pow::PoW, p3_poa::SimplePoa, Consensus, ConsensusAuthority,Header};

/// A Consensus engine that alternates back and forth between PoW and PoA sealed blocks.
struct AlternatingPowPoa{
    inner_pow   : PoW,
    inner_poa   : SimplePoa,
}

#[derive(Hash, Debug, PartialEq, Eq, Clone, Copy)]
struct AlternatingPowPoaDigest {
    authority: Option<ConsensusAuthority>,
    digest_for_threshold: Option<u64>
}

impl Consensus for AlternatingPowPoa{
    type Digest = AlternatingPowPoaDigest;

    fn validate(&self, parent_digest: &Self::Digest, header: &Header<Self::Digest>) -> bool {
		match header.height % 2 {

            1 => {
                 let parent_digest_pow = parent_digest.digest_for_threshold;
                 match parent_digest_pow {
                    Some(parent_digest_pow_value) => {
                        let pow_header =  Header::<u64>{
                            parent: header.parent,
                            height: header.height,
                            state_root: header.state_root,
                            extrinsics_root: header.extrinsics_root,
                            consensus_digest: header.consensus_digest.digest_for_threshold.unwrap(),
                         };
                         return self.inner_pow.validate( &0, &pow_header);
                    }
                    None => {
                        return false;
                    },
                 }       
            },

            _ => {
                let parent_digest_poa = parent_digest.authority;
                match parent_digest_poa {
                    Some(parent_digest_poa_value) => {
                        let poa_header =  Header::<ConsensusAuthority>{
                            parent: header.parent,
                            height: header.height,
                            state_root: header.state_root,
                            extrinsics_root: header.extrinsics_root,
                            consensus_digest: header.consensus_digest.authority.unwrap(),
                            };
                            return self.inner_poa.validate(&parent_digest_poa_value, &poa_header);
                    },
                    None => {
                        return false;
                    },   
                }
            }
        }
	}


	fn seal(
		&self,
		parent_digest: &Self::Digest,
		partial_header: Header<()>,
	) -> Option<Header<Self::Digest>> {

        match partial_header.height % 2 {

            1 => {
                let parent_digest_pow = parent_digest.digest_for_threshold;
                match parent_digest_pow {
                    Some(parent_digest_pow_value) => {
                        let pow_header = self.inner_pow.seal(&parent_digest_pow_value, partial_header);
                        return match pow_header {
                            Some (pw) => {
                                let header = Header::<Self::Digest> {
                                    parent: pw.parent,
                                    height: pw.height,
                                    state_root: pw.state_root,
                                    extrinsics_root: pw.extrinsics_root,
                                    consensus_digest: AlternatingPowPoaDigest {
                                            authority: None,
                                            digest_for_threshold : Some(pw.consensus_digest),
                                    }
                                };
                                Some(header)
                            },
                            None => { 
                                None 
                            }
                         }

                    }
                    None => {
                        None
                    }
                }
            },

            _ => {
                let parent_digest_poa = parent_digest.authority;
                match parent_digest_poa {
                    Some(parent_digets_pa_value) => {
                        let poa_header = self.inner_poa.seal(&parent_digets_pa_value, partial_header);
                        return match poa_header {
                            Some (pa) => {
                                let header = Header::<Self::Digest> {
                                    parent: pa.parent,
                                    height: pa.height,
                                    state_root: pa.state_root,
                                    extrinsics_root: pa.extrinsics_root,
                                    consensus_digest: AlternatingPowPoaDigest {
                                            authority: Some(pa.consensus_digest),
                                            digest_for_threshold : None,
                                    }
                                };
                                Some(header)
                            },
                            None => { 
                                None 
                            }
                        };
                    },
                    None => {
                        None
                    }
                }
            }
        }
    }

    fn create_default_instance() -> Self{
		return Self {
            inner_poa : SimplePoa::create_default_instance(),
            inner_pow : PoW::create_default_instance(),
		}
	}

}


#[test]


fn test_consensus_for_alternate_pow_poa() {

    type  PowPoaDigest = <AlternatingPowPoa as Consensus>::Digest;
    let mut chain:Vec< Header< PowPoaDigest > > = vec![];

    let genensis = Header::< PowPoaDigest > {
        parent: 0,
        height: 0,
        state_root: 0,
        extrinsics_root: 0,
        consensus_digest: PowPoaDigest {
                authority: Some(ConsensusAuthority::Bob),
                digest_for_threshold: Some(0),
            }
        };

    let genensis2 = Header::< PowPoaDigest > {
        parent: hash(&genensis),
        height: 1,
        state_root: 1,
        extrinsics_root: 0,
        consensus_digest: PowPoaDigest {
                authority: None,
                digest_for_threshold: Some(0),
            }
        };

    chain.push(genensis);
    chain.push(genensis2);
    
    let pow_poa_consensus = AlternatingPowPoa {
        inner_poa : SimplePoa {
                    authorities: vec![ConsensusAuthority::Alice,ConsensusAuthority::Bob,ConsensusAuthority::Charlie]
                },
        inner_pow : PoW {
            threshold: u64::max_value() / 100,
        }
    };

    for i in 2..10 {
        

        let partial_header =  Header::<()>{
            parent: hash(&chain[i-2]),
            height: i as u64,
            state_root: i as u64,
            extrinsics_root: hash(&vec![1+i,2+i,3+i]),
            consensus_digest: ()
        };

        let h = pow_poa_consensus.seal(&chain[i-2].consensus_digest, partial_header);
        match h {
            Some(h) => {
                chain.push(h);
                println!("{:?}",chain[i]);
            }
            None => {println!("{:?}",chain[0]);}
        }
        
    }

    let mut check = true;
    for i in 2..10 { 
        check &= pow_poa_consensus.validate(&chain[i-2].consensus_digest, &chain[i])
    }
    assert!(check);

}