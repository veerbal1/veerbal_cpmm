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
  getAccount,
} from "@solana/spl-token";
import { assert } from "chai";

describe("swap", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.VeerbalCpmm as anchor.Program<VeerbalCpmm>;
  const owner = anchor.Wallet.local().payer;

  let configPDA: PublicKey;
  let poolPDA: PublicKey;
  let token0Mint: PublicKey;
  let token1Mint: PublicKey;
  let vault0Pda: PublicKey;
  let vault1Pda: PublicKey;
  let authorityPda: PublicKey;
  let userToken0Ata: PublicKey;
  let userToken1Ata: PublicKey;
  // Unique index per run (for devnet compatibility)
  const configIndex = (Math.floor(Date.now() / 1000) + 4000) % 65535;

  before(async () => {
    // 1. Create config
    configPDA = get_amm_config_pda({
      index: configIndex,
      program_id: program.programId,
    });

    await program.methods
      .createConfig(
        configIndex,
        new anchor.BN(2500), // 0.25% trade fee
        new anchor.BN(0),
        new anchor.BN(100000), // 10% protocol fee (of trade fee)
        new anchor.BN(250000), // 25% fund fee (of trade fee)
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
    const lpMintPda = get_lp_mint_pda({ program_id: program.programId, pool: poolPDA });

    // 5. Create pool with initial liquidity (10B each)
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

    console.log("Pool ready for swap test:", poolPDA.toBase58());
  });

  it("swaps token0 for token1 (base input)", async () => {
    // Get balances before swap
    const user0Before = await getAccount(provider.connection, userToken0Ata);
    const user1Before = await getAccount(provider.connection, userToken1Ata);
    const vault0Before = await getAccount(provider.connection, vault0Pda);
    const vault1Before = await getAccount(provider.connection, vault1Pda);

    const amountIn = new anchor.BN(100_000_000); // 0.1 token
    const minimumAmountOut = new anchor.BN(1); // Min output (slippage tolerance)

    // Execute swap: token0 -> token1
    await program.methods
      .swap(amountIn, minimumAmountOut)
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

    // Verify balances after swap
    const user0After = await getAccount(provider.connection, userToken0Ata);
    const user1After = await getAccount(provider.connection, userToken1Ata);
    const vault0After = await getAccount(provider.connection, vault0Pda);
    const vault1After = await getAccount(provider.connection, vault1Pda);

    const token0Spent = user0Before.amount - user0After.amount;
    const token1Received = user1After.amount - user1Before.amount;

    console.log("Token0 spent:", token0Spent.toString());
    console.log("Token1 received:", token1Received.toString());

    assert.equal(token0Spent.toString(), amountIn.toString(), "Should spend exact input amount");
    assert(token1Received > 0, "Should receive some token1");

    // Verify vault changes
    assert(vault0After.amount > vault0Before.amount, "Vault0 should increase");
    assert(vault1After.amount < vault1Before.amount, "Vault1 should decrease");

    console.log("Swap base input successful!");
  });

  it("swaps token1 for token0 (base output)", async () => {
    // Get balances before swap
    const user0Before = await getAccount(provider.connection, userToken0Ata);
    const user1Before = await getAccount(provider.connection, userToken1Ata);

    const amountOut = new anchor.BN(50_000_000); // Want exactly 0.05 token0
    const maximumAmountIn = new anchor.BN(100_000_000); // Max willing to spend

    // Execute swap: token1 -> token0 (want exact output of token0)
    await program.methods
      .swapBaseOutput(amountOut, maximumAmountIn)
      .accounts({
        signer: owner.publicKey,
        poolState: poolPDA,
        ammConfig: configPDA,
        inputTokenMint: token1Mint,
        outputTokenMint: token0Mint,
        inputVault: vault1Pda,
        outputVault: vault0Pda,
        inputTokenAccount: userToken1Ata,
        outputTokenAccount: userToken0Ata,
        authority: authorityPda,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    // Verify balances after swap
    const user0After = await getAccount(provider.connection, userToken0Ata);
    const user1After = await getAccount(provider.connection, userToken1Ata);

    const token0Received = user0After.amount - user0Before.amount;
    const token1Spent = user1Before.amount - user1After.amount;

    console.log("Token1 spent:", token1Spent.toString());
    console.log("Token0 received:", token0Received.toString());

    assert.equal(token0Received.toString(), amountOut.toString(), "Should receive exact output amount");
    assert(token1Spent > 0, "Should spend some token1");
    assert(token1Spent <= BigInt(maximumAmountIn.toString()), "Should not exceed max input");

    console.log("Swap base output successful!");
  });
});
