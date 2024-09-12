import { Program, web3 } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';
import {
    PublicKey,
    SystemProgram,
    SYSVAR_RENT_PUBKEY,
} from '@solana/web3.js';
import {
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID
} from "@solana/spl-token";
import fs from 'fs';
import { GlobalPool, UserPairPool, UserPool } from './types';

export const METAPLEX = new web3.PublicKey('metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s');

const USER_POOL_SIZE = 5664;     // 8 + 5656
const USER_PAIR_POOL_SIZE = 6720;     // 8 + 6712
const GLOBAL_AUTHORITY_SEED = "global-authority";

const REWARD_TOKEN_MINT = new PublicKey("32CHtMAuGaCAZx8Rgp54jSFG3ihbpN5brSvRAWpwEHPv");
const PROGRAM_ID = "c7qTRH6XvUCfTUNPjyRotdET7vZ34HSJakwxW3ZdabW";

anchor.setProvider(anchor.Provider.local(web3.clusterApiUrl('mainnet-beta')));
const solConnection = anchor.getProvider().connection;
const payer = anchor.getProvider().wallet;

let rewardVault: PublicKey = null;
let program: Program = null;

// Configure the client to use the local cluster.
// const walletKeypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(path.resolve("/home/fury/.config/solana/id.json"), 'utf-8'))), { skipValidation: true });

const idl = JSON.parse(
    fs.readFileSync(__dirname + "/basc_staking.json", "utf8")
);

// Address of the deployed program.
const programId = new anchor.web3.PublicKey(PROGRAM_ID);

// Generate the program client from IDL.
program = new anchor.Program(idl, programId);
console.log('ProgramId: ', program.programId.toBase58());
const DECIMALS = 1_000_000_000;
const EPOCH = 1;          // 1 sec
const MATCHING_FACTOR = 125;        // 1.25x

const NORMAL_APE_REWARD = 115_741;         // 10 $DAB / day
const HIGH_APE_REWARD = 173_611;      // 15 $DAB / day
const SUPER_APE_REWARD = 1_157_407;      // 100 $DAB / day

const NORMAL_TIGER_REWARD = 23_148;         // 2 $DAB / day
const HIGH_TIGER_REWARD = 34_722;      // 3 $DAB / day
const SUPER_TIGER_REWARD = 231_481;      // 20 $DAB / day

const main = async () => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    console.log('GlobalAuthority: ', globalAuthority.toBase58());

    rewardVault = await getAssociatedTokenAccount(globalAuthority, REWARD_TOKEN_MINT);
    console.log('RewardVault: ', rewardVault.toBase58());
    console.log(await solConnection.getTokenAccountBalance(rewardVault));

    // await initProject();

    const globalPool: GlobalPool = await getGlobalState();
    console.log("globalPool =", globalPool.superAdmin.toBase58(), globalPool.totalStakedApeCount.toNumber(), globalPool.totalStakedTigerCount.toNumber(), globalPool.totalStakedPairCount.toNumber());

    // await initUserPool(payer.publicKey);
    // await initUserPairPool(payer.publicKey);

    // await stakeNft(payer.publicKey, new PublicKey('XMAAJ4vvAz2YRSRGpAKCdEk4HvEZ8CLZjsS6JFwv5CM'), 1);
    // await stakeNft(payer.publicKey, new PublicKey('14qYhMe3uZCkzAsuy9A3SBtxUprBdgizojWAMVtWBHcr'), 1);
    // await withdrawNft(payer.publicKey, new PublicKey('XMAAJ4vvAz2YRSRGpAKCdEk4HvEZ8CLZjsS6JFwv5CM'));
    // await withdrawNft(payer.publicKey, new PublicKey('14qYhMe3uZCkzAsuy9A3SBtxUprBdgizojWAMVtWBHcr'));
    // await claimReward(payer.publicKey, new PublicKey('25EKJfvmeeVpL2eBMTP1w8QMbo86sSxyxGKykZWwq1R7'));
    // await stakePair(payer.publicKey, new PublicKey('14qYhMe3uZCkzAsuy9A3SBtxUprBdgizojWAMVtWBHcr'), 5001, new PublicKey('XMAAJ4vvAz2YRSRGpAKCdEk4HvEZ8CLZjsS6JFwv5CM'), 1);
    // await withdrawPair(payer.publicKey, new PublicKey('14qYhMe3uZCkzAsuy9A3SBtxUprBdgizojWAMVtWBHcr'), new PublicKey('XMAAJ4vvAz2YRSRGpAKCdEk4HvEZ8CLZjsS6JFwv5CM'));
    // await claimPairReward(payer.publicKey, new PublicKey('14qYhMe3uZCkzAsuy9A3SBtxUprBdgizojWAMVtWBHcr'), new PublicKey('XMAAJ4vvAz2YRSRGpAKCdEk4HvEZ8CLZjsS6JFwv5CM'));

    // const stakedInfo = await getStakedNFTsFromWallet(new PublicKey('7eaQfTpHQGD6EnZNgLErkZy12NB7p5z3M8KHg3NyDBC9'));
    // console.log(stakedInfo);
    // const userPool: UserPool = await getUserPoolState(payer.publicKey);//new PublicKey('9rDdTaSR8F4iDteLsLGmTVuVP3uw1wx5uXRs7LdWCqXQ'));
    // console.log({
    //     owner: userPool.owner.toBase58(),
    //     stakedNfts: userPool.stakedNfts.slice(0, userPool.stakedCount.toNumber()).map((info) => {
    //         return {
    //             mint: info.mint.toBase58(),
    //             lastClaimedTime: info.lastClaimedTime.toNumber(),
    //             rank: info.rank.toNumber(),
    //             isApe: info.isApe,
    //         }
    //     }),
    //     stakedCount: userPool.stakedCount.toNumber(),
    //     remainingRewards: userPool.remainingRewards.toNumber(),
    //     lastRewardTime: (new Date(1000 * userPool.lastRewardTime.toNumber())).toLocaleString(),
    // });
    
    // const userPool: UserPairPool = await getUserPairPoolState(payer.publicKey);//new PublicKey('9rDdTaSR8F4iDteLsLGmTVuVP3uw1wx5uXRs7LdWCqXQ'));
    // console.log({
    //     owner: userPool.owner.toBase58(),
    //     stakedPairs: userPool.stakedPairs.slice(0, userPool.stakedPairCount.toNumber()).map((info) => {
    //         return {
    //             apeMint: info.apeMint.toBase58(),
    //             apeRank: info.apeRank.toNumber(),
    //             apeId: info.apeId.toNumber(),
    //             tigerMint: info.tigerMint.toBase58(),
    //             tigerRank: info.tigerRank.toNumber(),
    //             tigerId: info.tigerId.toNumber(),
    //             lastClaimedTime: info.lastClaimedTime.toNumber(),
    //         }
    //     }),
    //     stakedPairCount: userPool.stakedPairCount.toNumber(),
    //     remainingRewards: userPool.remainingRewards.toNumber(),
    //     lastRewardTime: (new Date(1000 * userPool.lastRewardTime.toNumber())).toLocaleString(),
    // });
    // console.log(await calculateAvailableReward(payer.publicKey))//, new PublicKey('25EKJfvmeeVpL2eBMTP1w8QMbo86sSxyxGKykZWwq1R7')));
    // console.log(await calculateAvailablePairReward(payer.publicKey, new PublicKey('25EKJfvmeeVpL2eBMTP1w8QMbo86sSxyxGKykZWwq1R7')));
    await getAllStakers();
    await getAllPairStakers();
};

export const getAllStakers = async () => {
    let poolAccounts = await solConnection.getProgramAccounts(
      program.programId,
      {
        filters: [
          {
            dataSize: USER_POOL_SIZE
          },
        ]
      }
    );
    
    console.log(`Encounter ${poolAccounts.length} Stakers`);
    
    let result: UserPool[] = [];

    try {
        for (let idx = 0; idx < poolAccounts.length; idx++) {
            let data = poolAccounts[idx].account.data;
            const owner = new PublicKey(data.slice(8, 40));

            let buf = data.slice(40, 48).reverse();
            let cnt = (new anchor.BN(buf));

            let stakedNfts = [];
            for (let i = 48, j = 0; i <= data.length - 16 && j < cnt.toNumber(); j++) {
                buf = data.slice(i, i + 32);
                let mint = (new PublicKey(buf));
                i += 32;
                buf = data.slice(i, i + 8).reverse();
                let time = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 8).reverse();
                let rank = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 8).reverse();
                let isApe = new anchor.BN(buf);
                i += 8;
                stakedNfts.push({mint, lastClaimedTime: time, rank, isApe});
            }

            buf = data.slice(data.length - 16, data.length - 8).reverse();
            let last = (new anchor.BN(buf));

            buf = data.slice(data.length - 8, data.length).reverse();
            let pending = (new anchor.BN(buf));

            result.push({
                owner,
                stakedCount: cnt,
                stakedNfts,
                lastRewardTime: last,
                remainingRewards: pending,
            });
        }
    } catch (e) {
        console.log(e);
        return {};
    }

    console.dir({
        count: poolAccounts.length,
        data: result.map((info: UserPool) => {
            return {
                owner: info.owner.toBase58(),
                stakedCount: info.stakedCount.toNumber(),
                stakedNfts: info.stakedNfts.slice(0, info.stakedCount.toNumber()).map((info) => {
                    return {
                        mint: info.mint.toBase58(),
                        lastClaimedTime: info.lastClaimedTime.toNumber(),
                        rank: info.rank.toNumber(),
                        isApe: info.isApe.toNumber(),
                    }
                }),
                remainingRewards: info.remainingRewards.toNumber(),
                lastRewardTime: (new Date(1000 * info.lastRewardTime.toNumber())).toLocaleString(),
            }
        })
    }, {depth: null})
};

export const getAllPairStakers = async () => {
    let poolAccounts = await solConnection.getProgramAccounts(
      program.programId,
      {
        filters: [
          {
            dataSize: USER_PAIR_POOL_SIZE
          },
        ]
      }
    );
    
    console.log(`Encounter ${poolAccounts.length} Pair Stakers`);
    
    let result: UserPairPool[] = [];

    try {
        for (let idx = 0; idx < poolAccounts.length; idx++) {
            let data = poolAccounts[idx].account.data;
            const owner = new PublicKey(data.slice(8, 40));

            let buf = data.slice(40, 48).reverse();
            let cnt = (new anchor.BN(buf));

            let stakedPairs = [];
            for (let i = 48, j = 0; i <= data.length - 16 && j < cnt.toNumber(); j++) {
                buf = data.slice(i, i + 32);
                let apeMint = (new PublicKey(buf));
                i += 32;
                buf = data.slice(i, i + 8).reverse();
                let apeRank = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 8).reverse();
                let apeId = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 32);
                let tigerMint = (new PublicKey(buf));
                i += 32;
                buf = data.slice(i, i + 8).reverse();
                let tigerRank = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 8).reverse();
                let tigerId = new anchor.BN(buf);
                i += 8;
                buf = data.slice(i, i + 8).reverse();
                let lastClaimedTime = new anchor.BN(buf);
                i += 8;
                stakedPairs.push({apeMint, apeRank, apeId, tigerMint, tigerRank, tigerId, lastClaimedTime});
            }

            buf = data.slice(data.length - 16, data.length - 8).reverse();
            let last = (new anchor.BN(buf));

            buf = data.slice(data.length - 8, data.length).reverse();
            let pending = (new anchor.BN(buf));

            result.push({
                owner,
                stakedPairCount: cnt,
                stakedPairs,
                lastRewardTime: last,
                remainingRewards: pending,
            });
        }
    } catch (e) {
        console.log(e);
        return {};
    }
    
    let pairIdList = [];
    let data = result.map((info: UserPairPool) => {
        return {
            owner: info.owner.toBase58(),
            stakedPairCount: info.stakedPairCount.toNumber(),
            stakedPairs: info.stakedPairs.slice(0, info.stakedPairCount.toNumber()).map((info) => {
                pairIdList.push(info.apeId.toNumber());
                return {
                    apeMint: info.apeMint.toBase58(),
                    apeRank: info.apeRank.toNumber(),
                    apeId: info.apeId.toNumber(),
                    tigerMint: info.tigerMint.toBase58(),
                    tigerRank: info.tigerRank.toNumber(),
                    tigerId: info.tigerId.toNumber(),
                    lastClaimedTime: info.lastClaimedTime.toNumber(),
                }
            }),
            remainingRewards: info.remainingRewards.toNumber(),
            lastRewardTime: (new Date(1000 * info.lastRewardTime.toNumber())).toLocaleString(),
        }
    })

    console.dir({
        count: poolAccounts.length,
        pairIdList,
        data,
    }, {depth: null})
};

export const getStakedNFTsFromWallet = async (address: PublicKey) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    console.log('GlobalAuthority: ', globalAuthority.toBase58());

    const userPool: UserPool = await getUserPoolState(address);
    return {
        holder: globalAuthority.toBase58(),
        stakedCount: userPool.stakedCount.toNumber(),
        stakedMints: userPool.stakedNfts.slice(0, userPool.stakedCount.toNumber()).map((info) => {
            return info.mint.toBase58();
        })
    }
};

export const initProject = async (
) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    const tx = await program.rpc.initialize(
        bump, {
        accounts: {
            admin: payer.publicKey,
            globalAuthority,
            rewardVault: rewardVault,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY,
        },
        signers: [],
    });
    await solConnection.confirmTransaction(tx, "confirmed");

    console.log("txHash =", tx);
    return false;
}

export const initUserPool = async (
    userAddress: PublicKey,
) => {
    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );

    console.log(USER_POOL_SIZE);
    let ix = SystemProgram.createAccountWithSeed({
        fromPubkey: userAddress,
        basePubkey: userAddress,
        seed: "user-pool",
        newAccountPubkey: userPoolKey,
        lamports: await solConnection.getMinimumBalanceForRentExemption(USER_POOL_SIZE),
        space: USER_POOL_SIZE,
        programId: program.programId,
    });

    const tx = await program.rpc.initializeUserPool(
        {
            accounts: {
                userPool: userPoolKey,
                owner: userAddress
            },
            instructions: [
                ix
            ],
            signers: []
        }
    );
    await solConnection.confirmTransaction(tx, "finalized");

    console.log("Your transaction signature", tx);
    let poolAccount = await program.account.userPool.fetch(userPoolKey);
    console.log('Owner of initialized pool = ', poolAccount.owner.toBase58());
}

export const initUserPairPool = async (
    userAddress: PublicKey,
) => {
    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pair-pool",
        program.programId,
    );

    console.log(USER_PAIR_POOL_SIZE);
    let ix = SystemProgram.createAccountWithSeed({
        fromPubkey: userAddress,
        basePubkey: userAddress,
        seed: "user-pair-pool",
        newAccountPubkey: userPoolKey,
        lamports: await solConnection.getMinimumBalanceForRentExemption(USER_PAIR_POOL_SIZE),
        space: USER_PAIR_POOL_SIZE,
        programId: program.programId,
    });

    const tx = await program.rpc.initializeUserPairPool(
        {
            accounts: {
                userPool: userPoolKey,
                owner: userAddress
            },
            instructions: [
                ix
            ],
            signers: []
        }
    );
    await solConnection.confirmTransaction(tx, "finalized");

    console.log("Your transaction signature", tx);
    let poolAccount = await program.account.userPairPool.fetch(userPoolKey);
    console.log('Owner of initialized pair pool = ', poolAccount.owner.toBase58());
}

export const stakeNft = async (userAddress: PublicKey, mint: PublicKey, rank: number) => {
    let userTokenAccount = await getAssociatedTokenAccount(userAddress, mint);
    let accountOfNFT = await getNFTTokenAccount(mint);
    if (userTokenAccount.toBase58() != accountOfNFT.toBase58()) {
        let nftOwner = await getOwnerOfNFT(mint);
        if (nftOwner.toBase58() == userAddress.toBase58()) userTokenAccount = accountOfNFT;
        else {
            console.log('Error: Nft is not owned by user');
            return;
        }
    }
    console.log("BASC NFT = ", mint.toBase58(), userTokenAccount.toBase58());

    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [mint]
    );

    console.log("Dest NFT Account = ", destinationAccounts[0].toBase58())
    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );

    let poolAccount = await solConnection.getAccountInfo(userPoolKey);
    if (poolAccount === null || poolAccount.data === null) {
        await initUserPool(userAddress);
    }

    const metadata = await getMetadata(mint);
    console.log("Metadata=", metadata.toBase58());
   
    const tx = await program.rpc.stakeNftToPool(
        bump, new anchor.BN(rank), {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            userTokenAccount,
            destNftTokenAccount: destinationAccounts[0],
            nftMint: mint,
            mintMetadata: metadata,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METAPLEX,
        },
        instructions: [
            ...instructions,
        ],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "singleGossip");
}

export const withdrawNft = async (userAddress: PublicKey, mint: PublicKey) => {
    let retUser = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [mint]
    );
    let userTokenAccount = retUser.destinationAccounts[0];
    console.log("BASC NFT = ", mint.toBase58(), userTokenAccount.toBase58());

    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [mint]
    );

    console.log("Dest NFT Account = ", destinationAccounts[0].toBase58());

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );

    let ret = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [REWARD_TOKEN_MINT]
    );

    const tx = await program.rpc.withdrawNftFromPool(
        bump, {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            userTokenAccount,
            destNftTokenAccount: destinationAccounts[0],
            nftMint: mint,
            rewardVault,
            userRewardAccount: ret.destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
            ...retUser.instructions,
            ...ret.instructions,
        ],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "singleGossip");
}

export const claimReward = async (userAddress: PublicKey, mint?: PublicKey) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    console.log("globalAuthority =", globalAuthority.toBase58());

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [REWARD_TOKEN_MINT]
    );

    console.log("User Reward Account = ", destinationAccounts[0].toBase58());
    let tx = await program.rpc.claimReward(
        bump, mint ? mint : null, {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            rewardVault,
            userRewardAccount: destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
            ...instructions,
        ],
        signers: []
    });

    console.log("Your transaction signature", tx);
    await solConnection.confirmTransaction(tx, "singleGossip");

    console.log(await solConnection.getTokenAccountBalance(destinationAccounts[0]));
}

export const stakePair = async (userAddress: PublicKey, ape_mint: PublicKey, ape_rank: number, tiger_mint: PublicKey, tiger_rank: number) => {
    let userApeTokenAccount = await getAssociatedTokenAccount(userAddress, ape_mint);
    let apeAccountOfNFT = await getNFTTokenAccount(ape_mint);
    if (userApeTokenAccount.toBase58() != apeAccountOfNFT.toBase58()) {
        let nftOwner = await getOwnerOfNFT(ape_mint);
        if (nftOwner.toBase58() == userAddress.toBase58()) userApeTokenAccount = apeAccountOfNFT;
        else {
            console.log('Error: Ape Nft is not owned by user');
            return;
        }
    }
    let userTigerTokenAccount = await getAssociatedTokenAccount(userAddress, tiger_mint);
    let tigerAccountOfNFT = await getNFTTokenAccount(tiger_mint);
    if (userTigerTokenAccount.toBase58() != tigerAccountOfNFT.toBase58()) {
        let nftOwner = await getOwnerOfNFT(tiger_mint);
        if (nftOwner.toBase58() == userAddress.toBase58()) userTigerTokenAccount = tigerAccountOfNFT;
        else {
            console.log('Error: Tiger Nft is not owned by user');
            return;
        }
    }
    console.log("BASC Ape NFT = ", ape_mint.toBase58(), userApeTokenAccount.toBase58());
    console.log("BASC Tiger NFT = ", tiger_mint.toBase58(), userTigerTokenAccount.toBase58());

    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let retApe = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [ape_mint]
    );
    console.log("Dest Ape NFT Account = ", retApe.destinationAccounts[0].toBase58())
    
    let retTiger = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [tiger_mint]
    );
    console.log("Dest Tiger NFT Account = ", retTiger.destinationAccounts[0].toBase58())
    
    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pair-pool",
        program.programId,
    );

    let poolAccount = await solConnection.getAccountInfo(userPoolKey);
    if (poolAccount === null || poolAccount.data === null) {
        await initUserPairPool(userAddress);
    }

    const apeMetadata = await getMetadata(ape_mint);
    console.log("ApeMetadata=", apeMetadata.toBase58());
    
    const tigerMetadata = await getMetadata(tiger_mint);
    console.log("TigerMetadata=", tigerMetadata.toBase58());
   
    const tx = await program.rpc.stakeNftToPairPool(
        bump, new anchor.BN(ape_rank), new anchor.BN(tiger_rank), {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            userApeTokenAccount,
            destApeNftTokenAccount: retApe.destinationAccounts[0],
            apeNftMint: ape_mint,
            apeMintMetadata: apeMetadata,
            userTigerTokenAccount,
            destTigerNftTokenAccount: retTiger.destinationAccounts[0],
            tigerNftMint: tiger_mint,
            tigerMintMetadata: tigerMetadata,
            tokenProgram: TOKEN_PROGRAM_ID,
            tokenMetadataProgram: METAPLEX,
        },
        instructions: [
            ...retApe.instructions,
            ...retTiger.instructions,
        ],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "singleGossip");
}

export const withdrawPair = async (userAddress: PublicKey, ape_mint: PublicKey, tiger_mint: PublicKey) => {
    let retUserApe = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [ape_mint]
    );
    let userApeTokenAccount = retUserApe.destinationAccounts[0];
    console.log("BASC Ape NFT = ", ape_mint.toBase58(), userApeTokenAccount.toBase58());
    
    let retUserTiger = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [tiger_mint]
    );
    let userTigerTokenAccount = retUserTiger.destinationAccounts[0];
    console.log("BASC Tiger NFT = ", tiger_mint.toBase58(), userTigerTokenAccount.toBase58());

    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    let retApe = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [ape_mint]
    );
    console.log("Dest Ape NFT Account = ", retApe.destinationAccounts[0].toBase58());

    let retTiger = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        globalAuthority,
        [tiger_mint]
    );
    console.log("Dest Tiger NFT Account = ", retTiger.destinationAccounts[0].toBase58());

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pair-pool",
        program.programId,
    );

    let ret = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [REWARD_TOKEN_MINT]
    );

    const tx = await program.rpc.withdrawNftFromPairPool(
        bump, {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            rewardVault,
            userApeTokenAccount,
            destApeNftTokenAccount: retApe.destinationAccounts[0],
            apeNftMint: ape_mint,
            userTigerTokenAccount,
            destTigerNftTokenAccount: retTiger.destinationAccounts[0],
            tigerNftMint: tiger_mint,
            userRewardAccount: ret.destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
            ...retUserApe.instructions,
            ...retUserTiger.instructions,
            ...ret.instructions,
        ],
        signers: [],
    }
    );
    await solConnection.confirmTransaction(tx, "singleGossip");
}

export const claimPairReward = async (userAddress: PublicKey, ape_mint?: PublicKey, tiger_mint?: PublicKey) => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );

    console.log("globalAuthority =", globalAuthority.toBase58());

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pair-pool",
        program.programId,
    );

    let { instructions, destinationAccounts } = await getATokenAccountsNeedCreate(
        solConnection,
        userAddress,
        userAddress,
        [REWARD_TOKEN_MINT]
    );

    console.log("User Reward Account = ", destinationAccounts[0].toBase58());
    let tx = await program.rpc.claimPairReward(
        bump, ape_mint ? ape_mint : null, tiger_mint ? tiger_mint : null, {
        accounts: {
            owner: userAddress,
            userPool: userPoolKey,
            globalAuthority,
            rewardVault,
            userRewardAccount: destinationAccounts[0],
            tokenProgram: TOKEN_PROGRAM_ID,
        },
        instructions: [
            ...instructions,
        ],
        signers: []
    });

    console.log("Your transaction signature", tx);
    await solConnection.confirmTransaction(tx, "singleGossip");

    console.log(await solConnection.getTokenAccountBalance(destinationAccounts[0]));
}

export const calculateAvailableReward = async (userAddress: PublicKey, mint?: PublicKey) => {
    const userPool: UserPool = await getUserPoolState(userAddress);
    const userPoolInfo = {
        // ...userPool,
        owner: userPool.owner.toBase58(),
        stakedNfts: userPool.stakedNfts.slice(0, userPool.stakedCount.toNumber()).map((info) => {
            return {
                // ...info,
                mint: info.mint.toBase58(),
                lastClaimedTime: info.lastClaimedTime.toNumber(),
                isApe: (new anchor.BN(info.isApe)).toNumber(),
                rank: info.rank.toNumber(),
            }
        }),
        stakedCount: userPool.stakedCount.toNumber(),
        remainingRewards: userPool.remainingRewards.toNumber(),
        lastRewardTime: (new Date(1000 * userPool.lastRewardTime.toNumber())).toLocaleString(),
    };
    console.log(userPoolInfo);

    let now = Math.floor(Date.now() / 1000);
    let totalReward = 0;
    const mintAddress: string = mint ? mint.toBase58() : "";
    console.log(`Now: ${now} Last_Reward_Time: ${userPool.lastRewardTime.toNumber()} MintAddress: ${mintAddress}`);
    for (let i = 0; i < userPoolInfo.stakedCount; i++) {
        if (mint) {
            if (userPoolInfo.stakedNfts[i].mint != mintAddress) continue;
        }

        let lastRewardTime = userPool.lastRewardTime.toNumber();
        if (lastRewardTime < userPoolInfo.stakedNfts[i].lastClaimedTime) {
            lastRewardTime = userPoolInfo.stakedNfts[i].lastClaimedTime;
        }

        let rewardAmount;
        if (userPoolInfo.stakedNfts[i].isApe == 1) {
            if (userPoolInfo.stakedNfts[i].rank < 13) rewardAmount = SUPER_APE_REWARD;
            else if (userPoolInfo.stakedNfts[i].rank < 1000) rewardAmount = HIGH_APE_REWARD;
            else rewardAmount = NORMAL_APE_REWARD;
        } else {
            if (userPoolInfo.stakedNfts[i].rank < 5013) rewardAmount = SUPER_TIGER_REWARD;
            else if (userPoolInfo.stakedNfts[i].rank < 6000) rewardAmount = HIGH_TIGER_REWARD;
            else rewardAmount = NORMAL_TIGER_REWARD;
        }

        let reward = 0;
        reward = (Math.floor((now - lastRewardTime) / EPOCH)) * rewardAmount;

        if (mint) return reward / DECIMALS;

        totalReward += reward;
    }

    // Did not find the Mint Address
    if (mint) {
        console.log("Can't find the Staked NFT Mint", mintAddress);
        return 0;
    }

    totalReward += userPoolInfo.remainingRewards;
    return totalReward / DECIMALS;
};

export const calculateAvailablePairReward = async (userAddress: PublicKey, mint?: PublicKey) => {
    let userPool: UserPairPool = await getUserPairPoolState(userAddress);
    const userPoolInfo = {
        owner: userPool.owner.toBase58(),
        stakedPairs: userPool.stakedPairs.slice(0, userPool.stakedPairCount.toNumber()).map((info) => {
            return {
                apeMint: info.apeMint.toBase58(),
                apeRank: info.apeRank.toNumber(),
                apeId: info.apeId.toNumber(),
                tigerMint: info.tigerMint.toBase58(),
                tigerRank: info.tigerRank.toNumber(),
                tigerId: info.tigerId.toNumber(),
                lastClaimedTime: info.lastClaimedTime.toNumber(),
            }
        }),
        stakedPairCount: userPool.stakedPairCount.toNumber(),
        remainingRewards: userPool.remainingRewards.toNumber(),
        lastRewardTime: (new Date(1000 * userPool.lastRewardTime.toNumber())).toLocaleString(),
    };
    console.log("UserPairPoolInfo", userPoolInfo);

    let now = Math.floor(Date.now() / 1000);
    let totalReward = 0;
    const mintAddress: string = mint ? mint.toBase58() : "";
    console.log(`Now: ${now} Last_Reward_Time: ${userPool.lastRewardTime.toNumber()} MintAddress: ${mintAddress}`);
    for (let i = 0; i < userPoolInfo.stakedPairCount; i++) {
        if (mint) {
            if (userPoolInfo.stakedPairs[i].apeMint != mintAddress && userPoolInfo.stakedPairs[i].tigerMint != mintAddress) continue;
        }

        let lastRewardTime = userPool.lastRewardTime.toNumber();
        if (lastRewardTime < userPoolInfo.stakedPairs[i].lastClaimedTime) {
            lastRewardTime = userPoolInfo.stakedPairs[i].lastClaimedTime;
        }

        let apeRewardAmount, tigerRewardAmount;
        if (userPoolInfo.stakedPairs[i].apeRank < 13) apeRewardAmount = SUPER_APE_REWARD;
        else if (userPoolInfo.stakedPairs[i].apeRank < 1000) apeRewardAmount = HIGH_APE_REWARD;
        else apeRewardAmount = NORMAL_APE_REWARD;

        if (userPoolInfo.stakedPairs[i].tigerRank < 5013) tigerRewardAmount = SUPER_TIGER_REWARD;
        else if (userPoolInfo.stakedPairs[i].tigerRank < 6000) tigerRewardAmount = HIGH_TIGER_REWARD;
        else tigerRewardAmount = NORMAL_TIGER_REWARD;

        let reward = 0, factor = 100;
        if (userPoolInfo.stakedPairs[i].apeId == userPoolInfo.stakedPairs[i].tigerId) factor = MATCHING_FACTOR;
        reward = Math.floor((Math.floor((now - lastRewardTime) / EPOCH)) * (apeRewardAmount + tigerRewardAmount) * factor / 100);

        if (mint) return reward / DECIMALS;

        totalReward += reward;
    }

    // Did not find the Mint Address
    if (mint) {
        console.log("Can't find the Staked NFT Mint", mintAddress);
        return 0;
    }

    totalReward += userPoolInfo.remainingRewards;
    return totalReward / DECIMALS;
};

export const getGlobalState = async (
): Promise<GlobalPool | null> => {
    const [globalAuthority, bump] = await PublicKey.findProgramAddress(
        [Buffer.from(GLOBAL_AUTHORITY_SEED)],
        program.programId
    );
    try {
        let globalState = await program.account.globalPool.fetch(globalAuthority);
        return globalState as GlobalPool;
    } catch {
        return null;
    }
}

export const getUserPoolState = async (
    userAddress: PublicKey
): Promise<UserPool | null> => {
    if (!userAddress) return null;

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pool",
        program.programId,
    );
    console.log('User Pool: ', userPoolKey.toBase58());
    try {
        let poolState = await program.account.userPool.fetch(userPoolKey);
        return poolState as UserPool;
    } catch {
        return null;
    }
}

export const getUserPairPoolState = async (
    userAddress: PublicKey
): Promise<UserPairPool | null> => {
    if (!userAddress) return null;

    let userPoolKey = await PublicKey.createWithSeed(
        userAddress,
        "user-pair-pool",
        program.programId,
    );
    console.log('User Pair Pool: ', userPoolKey.toBase58());
    try {
        let poolState = await program.account.userPairPool.fetch(userPoolKey);
        return poolState as UserPairPool;
    } catch {
        return null;
    }
}

const getOwnerOfNFT = async (nftMintPk : PublicKey) : Promise<PublicKey> => {
    let tokenAccountPK = await getNFTTokenAccount(nftMintPk);
    let tokenAccountInfo = await solConnection.getAccountInfo(tokenAccountPK);
    
    console.log("nftMintPk=", nftMintPk.toBase58());
    console.log("tokenAccountInfo =", tokenAccountInfo);
  
    if (tokenAccountInfo && tokenAccountInfo.data ) {
      let ownerPubkey = new PublicKey(tokenAccountInfo.data.slice(32, 64))
      console.log("ownerPubkey=", ownerPubkey.toBase58());
      return ownerPubkey;
    }
    return new PublicKey("");
}
  
const getNFTTokenAccount = async (nftMintPk : PublicKey) : Promise<PublicKey> => {
    console.log("getNFTTokenAccount nftMintPk=", nftMintPk.toBase58());
    let tokenAccount = await solConnection.getProgramAccounts(
      TOKEN_PROGRAM_ID,
      {
        filters: [
          {
            dataSize: 165
          },
          {
            memcmp: {
              offset: 64,
              bytes: '2'
            }
          },
          {
            memcmp: {
              offset: 0,
              bytes: nftMintPk.toBase58()
            }
          },
        ]
      }
    );
    return tokenAccount[0].pubkey;
}

const getAssociatedTokenAccount = async (ownerPubkey: PublicKey, mintPk: PublicKey): Promise<PublicKey> => {
    let associatedTokenAccountPubkey = (await PublicKey.findProgramAddress(
        [
            ownerPubkey.toBuffer(),
            TOKEN_PROGRAM_ID.toBuffer(),
            mintPk.toBuffer(), // mint address
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];
    return associatedTokenAccountPubkey;
}

export const getATokenAccountsNeedCreate = async (
    connection: anchor.web3.Connection,
    walletAddress: anchor.web3.PublicKey,
    owner: anchor.web3.PublicKey,
    nfts: anchor.web3.PublicKey[],
) => {
    let instructions = [], destinationAccounts = [];
    for (const mint of nfts) {
        const destinationPubkey = await getAssociatedTokenAccount(owner, mint);
        let response = await connection.getAccountInfo(destinationPubkey);
        if (!response) {
            const createATAIx = createAssociatedTokenAccountInstruction(
                destinationPubkey,
                walletAddress,
                owner,
                mint,
            );
            instructions.push(createATAIx);
        }
        destinationAccounts.push(destinationPubkey);
        if (walletAddress != owner) {
            const userAccount = await getAssociatedTokenAccount(walletAddress, mint);
            response = await connection.getAccountInfo(userAccount);
            if (!response) {
                const createATAIx = createAssociatedTokenAccountInstruction(
                    userAccount,
                    walletAddress,
                    walletAddress,
                    mint,
                );
                instructions.push(createATAIx);
            }
        }
    }
    return {
        instructions,
        destinationAccounts,
    };
}

export const createAssociatedTokenAccountInstruction = (
    associatedTokenAddress: anchor.web3.PublicKey,
    payer: anchor.web3.PublicKey,
    walletAddress: anchor.web3.PublicKey,
    splTokenMintAddress: anchor.web3.PublicKey
) => {
    const keys = [
        { pubkey: payer, isSigner: true, isWritable: true },
        { pubkey: associatedTokenAddress, isSigner: false, isWritable: true },
        { pubkey: walletAddress, isSigner: false, isWritable: false },
        { pubkey: splTokenMintAddress, isSigner: false, isWritable: false },
        {
            pubkey: anchor.web3.SystemProgram.programId,
            isSigner: false,
            isWritable: false,
        },
        { pubkey: TOKEN_PROGRAM_ID, isSigner: false, isWritable: false },
        {
            pubkey: anchor.web3.SYSVAR_RENT_PUBKEY,
            isSigner: false,
            isWritable: false,
        },
    ];
    return new anchor.web3.TransactionInstruction({
        keys,
        programId: ASSOCIATED_TOKEN_PROGRAM_ID,
        data: Buffer.from([]),
    });
}

/** Get metaplex mint metadata account address */
export const getMetadata = async (mint: PublicKey): Promise<PublicKey> => {
    return (
        await PublicKey.findProgramAddress([Buffer.from('metadata'), METAPLEX.toBuffer(), mint.toBuffer()], METAPLEX)
    )[0];
};

main();