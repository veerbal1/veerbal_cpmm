import * as anchor from "@coral-xyz/anchor";
import { VeerbalCpmm } from "../target/types/veerbal_cpmm";
import { PublicKey } from "@solana/web3.js";
import {
  get_amm_config_pda,
  get_pool_pda,
  get_vault_pda,
  get_lp_mint_pda,
  get_authority_pda,
} from "./utils";
import { SYSTEM_PROGRAM_ID } from "@coral-xyz/anchor/dist/cjs/native/system";
import {
  createMint,
  getAssociatedTokenAddressSync,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("collect-fees", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.VeerbalCpmm as anchor.Program<VeerbalCpmm>;
  const owner = anchor.Wallet.local().payer; // This is also protocol_owner and fund_owner

  let configPDA: PublicKey;
  let poolPDA: PublicKey;
  let token0Mint: PublicKey;
  let token1Mint: PublicKey;
  let vault0Pda: PublicKey;
  let vault1Pda: PublicKey;
  let authorityPda: PublicKey;
  let userToken0Ata: PublicKey;
  let userToken1Ata: PublicKey;
  const configIndex = 5; // Different index from other tests

  before(async () => {
    // 1. Create config - owner is protocol_owner and fund_owner
    configPDA = get_amm_config_pda({
      index: configIndex,
      program_id: program.programId,
    });

    await program.methods
      .createConfig(
        configIndex,
        new anchor.BN(25000), // 2.5% trade fee (higher to accumulate more fees)
        new anchor.BN(100000), // 10% creator fee
        new anchor.BN(200000), // 20% protocol fee
        new anchor.BN(300000), // 30% fund fee
        new anchor.BN(0)
      )
      .accounts({
        owner: owner.publicKey,
        ammConfig: configPDA,
        systemProgram: SYSTEM_PROGRAM_ID,
      })
      .signers([owner])
      .rpc();

    // 2. Create mints
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
    [token0Mint, token1Mint] =
      mintA.toBuffer().compare(mintB.toBuffer()) < 0
        ? [mintA, mintB]
        : [mintB, mintA];

    // 3. Create ATAs and fund them
    const token0Account = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      owner,
      token0Mint,
      owner.publicKey
    );
    const token1Account = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      owner,
      token1Mint,
      owner.publicKey
    );
    userToken0Ata = token0Account.address;
    userToken1Ata = token1Account.address;

    await mintTo(
      provider.connection,
      owner,
      token0Mint,
      userToken0Ata,
      owner,
      100_000_000_000
    );
    await mintTo(
      provider.connection,
      owner,
      token1Mint,
      userToken1Ata,
      owner,
      100_000_000_000
    );

    // 4. Derive pool PDAs
    poolPDA = get_pool_pda({
      program_id: program.programId,
      config_pda: configPDA,
      mint0: token0Mint,
      mint1: token1Mint,
    });
    vault0Pda = get_vault_pda({
      program_id: program.programId,
      pool: poolPDA,
      mint: token0Mint,
    });
    vault1Pda = get_vault_pda({
      program_id: program.programId,
      pool: poolPDA,
      mint: token1Mint,
    });
    authorityPda = get_authority_pda({ program_id: program.programId });

    // 5. Create pool
    await program.methods
      .createPool(
        configIndex,
        new anchor.BN(10_000_000_000),
        new anchor.BN(10_000_000_000),
        new anchor.BN(0)
      )
      .accounts({
        creator: owner.publicKey,
        ammConfig: configPDA,
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        creatorToken0: userToken0Ata,
        creatorToken1: userToken1Ata,
        feeReceiver: owner.publicKey,
      } as any)
      .signers([owner])
      .rpc();

    // Wait for pool to open
    await new Promise((resolve) => setTimeout(resolve, 3000));

    // 6. Execute a swap to accumulate fees
    const amountIn = new anchor.BN(1_000_000_000); // 1B tokens (2.5% = 25M fees)
    await program.methods
      .swap(amountIn, new anchor.BN(1))
      .accounts({
        signer: owner.publicKey,
        poolState: poolPDA,
        ammConfig: configPDA,
        inputTokenMint: token0Mint,
        outputTokenMint: token1Mint,
        inputVault: vault0Pda,
        outputVault: vault1Pda,
        inputTokenAccount: userToken0Ata,
        outputTokenAccount: userToken1Ata,
        authority: authorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    console.log("Pool ready with accumulated fees from swap");
  });

  it("collects protocol fees", async () => {
    // Check pool state before
    const poolBefore = await program.account.poolState.fetch(poolPDA);
    console.log("Protocol fee token0 before:", poolBefore.protocolToken0Fee.toString());
    console.log("Protocol fee token1 before:", poolBefore.protocolToken1Fee.toString());

    // Collect protocol fees
    await program.methods
      .collectProtocolFee()
      .accounts({
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        poolState: poolPDA,
        ammConfig: configPDA,
        token0Vault: vault0Pda,
        token1Vault: vault1Pda,
        authority: authorityPda,
        receiverToken0Account: userToken0Ata,
        receiverToken1Account: userToken1Ata,
        owner: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    // Check pool state after
    const poolAfter = await program.account.poolState.fetch(poolPDA);
    console.log("Protocol fee token0 after:", poolAfter.protocolToken0Fee.toString());
    console.log("Protocol fee token1 after:", poolAfter.protocolToken1Fee.toString());

    assert.equal(poolAfter.protocolToken0Fee.toNumber(), 0, "Protocol fee token0 should be 0");
    assert.equal(poolAfter.protocolToken1Fee.toNumber(), 0, "Protocol fee token1 should be 0");

    console.log("Protocol fee collection successful!");
  });

  it("collects fund fees", async () => {
    // Execute another swap to accumulate more fees
    await program.methods
      .swap(new anchor.BN(500_000_000), new anchor.BN(1))
      .accounts({
        signer: owner.publicKey,
        poolState: poolPDA,
        ammConfig: configPDA,
        inputTokenMint: token0Mint,
        outputTokenMint: token1Mint,
        inputVault: vault0Pda,
        outputVault: vault1Pda,
        inputTokenAccount: userToken0Ata,
        outputTokenAccount: userToken1Ata,
        authority: authorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    const poolBefore = await program.account.poolState.fetch(poolPDA);
    console.log("Fund fee token0 before:", poolBefore.fundToken0Fee.toString());

    // Collect fund fees
    await program.methods
      .collectFundFee()
      .accounts({
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        poolState: poolPDA,
        ammConfig: configPDA,
        token0Vault: vault0Pda,
        token1Vault: vault1Pda,
        authority: authorityPda,
        receiverToken0Account: userToken0Ata,
        receiverToken1Account: userToken1Ata,
        owner: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    const poolAfter = await program.account.poolState.fetch(poolPDA);
    assert.equal(poolAfter.fundToken0Fee.toNumber(), 0, "Fund fee token0 should be 0");
    assert.equal(poolAfter.fundToken1Fee.toNumber(), 0, "Fund fee token1 should be 0");

    console.log("Fund fee collection successful!");
  });

  it("collects creator fees", async () => {
    // Execute another swap to accumulate creator fees
    await program.methods
      .swap(new anchor.BN(500_000_000), new anchor.BN(1))
      .accounts({
        signer: owner.publicKey,
        poolState: poolPDA,
        ammConfig: configPDA,
        inputTokenMint: token0Mint,
        outputTokenMint: token1Mint,
        inputVault: vault0Pda,
        outputVault: vault1Pda,
        inputTokenAccount: userToken0Ata,
        outputTokenAccount: userToken1Ata,
        authority: authorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    const poolBefore = await program.account.poolState.fetch(poolPDA);
    console.log("Creator fee token0 before:", poolBefore.creatorToken0Fee.toString());

    // Collect creator fees (creator is the pool creator, which is owner)
    await program.methods
      .collectCreatorFee()
      .accounts({
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        poolState: poolPDA,
        token0Vault: vault0Pda,
        token1Vault: vault1Pda,
        authority: authorityPda,
        receiverToken0Account: userToken0Ata,
        receiverToken1Account: userToken1Ata,
        creator: owner.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        systemProgram: SYSTEM_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    const poolAfter = await program.account.poolState.fetch(poolPDA);
    assert.equal(poolAfter.creatorToken0Fee.toNumber(), 0, "Creator fee token0 should be 0");
    assert.equal(poolAfter.creatorToken1Fee.toNumber(), 0, "Creator fee token1 should be 0");

    console.log("Creator fee collection successful!");
  });
});
