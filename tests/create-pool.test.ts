import * as anchor from "@coral-xyz/anchor";
import { VeerbalCpmm } from "../target/types/veerbal_cpmm";
import { PublicKey } from "@solana/web3.js";
import {
  get_amm_config_pda,
  get_authority_pda,
  get_lp_mint_pda,
  get_pool_pda,
  get_vault_pda,
} from "./utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
} from "@solana/spl-token";

describe("create_pool", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.VeerbalCpmm as anchor.Program<VeerbalCpmm>;
  const owner = anchor.Wallet.local().payer;

  let configPDA: PublicKey;
  const configIndex = 1;
  let token0Mint: PublicKey;
  let token1Mint: PublicKey;

  // Create ATAs for creator
  let creatorToken0Ata: PublicKey;
  let creatorToken1Ata: PublicKey;

  before(async () => {
    configPDA = get_amm_config_pda({
      index: configIndex,
      program_id: program.programId,
    });

    await program.methods
      .createConfig(
        configIndex,
        new anchor.BN(2500), // trade_fee_rate
        new anchor.BN(0), // creator_fee_rate
        new anchor.BN(100000), // protocol_fee_rate
        new anchor.BN(250000), // fund_fee_rate
        new anchor.BN(0) // create_pool_fee (0 for testing)
      )
      .accounts({
        owner: owner.publicKey,
        ammConfig: configPDA,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();
    console.log("Config created:", configPDA.toBase58());

    const mintA = await createMint(
      provider.connection,
      owner,
      owner.publicKey,
      null,
      9
    );

    const mintB = await createMint(
      provider.connection,
      owner,
      owner.publicKey,
      null,
      9
    );

    if (mintA.toBuffer().compare(mintB.toBuffer()) < 0) {
      token0Mint = mintA;
      token1Mint = mintB;
    } else {
      token0Mint = mintB;
      token1Mint = mintA;
    }

    console.log("Token0:", token0Mint.toBase58());
    console.log("Token1:", token1Mint.toBase58());

    const token0Account = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      owner,
      token0Mint,
      owner.publicKey
    );
    creatorToken0Ata = token0Account.address;
    const token1Account = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      owner,
      token1Mint,
      owner.publicKey
    );
    creatorToken1Ata = token1Account.address;
    // Mint tokens to creator (10 billion each)
    await mintTo(
      provider.connection,
      owner,
      token0Mint,
      creatorToken0Ata,
      owner,
      10_000_000_000
    );
    await mintTo(
      provider.connection,
      owner,
      token1Mint,
      creatorToken1Ata,
      owner,
      10_000_000_000
    );
    console.log("Creator Token0 ATA:", creatorToken0Ata.toBase58());
    console.log("Creator Token1 ATA:", creatorToken1Ata.toBase58());
  });

  it("creates pool with initial liquidity", async () => {
    const poolPda = get_pool_pda({
      program_id: program.programId,
      config_pda: configPDA,
      mint0: token0Mint,
      mint1: token1Mint,
    });

    const vault0Pda = get_vault_pda({
      program_id: program.programId,
      pool: poolPda,
      mint: token0Mint,
    });

    const vault1Pda = get_vault_pda({
      program_id: program.programId,
      pool: poolPda,
      mint: token1Mint,
    });

    const lpMintPda = get_lp_mint_pda({
      program_id: program.programId,
      pool: poolPda,
    });

    const authorityPda = get_authority_pda({
      program_id: program.programId,
    });

    const creatorLpAta = getAssociatedTokenAddressSync(
      lpMintPda,
      owner.publicKey
    );

    await program.methods
      .createPool(
        configIndex,
        new anchor.BN(1_000_000_000),
        new anchor.BN(1_000_000_000),
        new anchor.BN(0)
      )
      .accounts({
        creator: owner.publicKey,
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        ammConfig: configPDA,
        creatorToken0: creatorToken0Ata,
        creatorToken1: creatorToken1Ata,
        feeReceiver: owner.publicKey,
      } as any)
      .signers([owner])
      .rpc();

    console.log("Pool created:", poolPda.toBase58());
  });
});
