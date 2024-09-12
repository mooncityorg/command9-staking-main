import * as anchor from '@project-serum/anchor';
import {PublicKey} from '@solana/web3.js';

export interface GlobalPool {
    superAdmin: PublicKey,             // 32
    totalStakedApeCount: anchor.BN,    // 8
    totalStakedTigerCount: anchor.BN,  // 8
    totalStakedPairCount: anchor.BN,  // 8
}

export interface StakedData {
    mint: PublicKey,             // 32
    lastClaimedTime: anchor.BN,  // 8
    rank: anchor.BN,             // 8
    isApe: anchor.BN,            // 8
}

export interface UserPool {
    // 8 + 5656
    owner: PublicKey,              // 32
    stakedCount: anchor.BN,        // 8
    stakedNfts: StakedData[],      // 56 * 100
    lastRewardTime: anchor.BN,     // 8
    remainingRewards: anchor.BN,   // 8
}

export interface PairStakedData {
    apeMint: PublicKey,             // 32
    apeRank: anchor.BN,             // 8
    apeId: anchor.BN,               // 8
    tigerMint: PublicKey,           // 32
    tigerRank: anchor.BN,           // 8
    tigerId: anchor.BN,             // 8
    lastClaimedTime: anchor.BN,     // 8
}

export interface UserPairPool {
    // 8 + 6712
    owner: PublicKey,                   // 32
    stakedPairCount: anchor.BN,         // 8
    stakedPairs: PairStakedData[],      // 104 * 64
    lastRewardTime: anchor.BN,          // 8
    remainingRewards: anchor.BN,        // 8
}
