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

describe("deposit", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider() as anchor.AnchorProvider;
  const program = anchor.workspace.VeerbalCpmm as anchor.Program<VeerbalCpmm>;
  const owner = anchor.Wallet.local().payer;

  let configPDA: PublicKey;
  let poolPDA: PublicKey;
  let token0Mint: PublicKey;
  let token1Mint: PublicKey;
  let lpMintPda: PublicKey;
  let vault0Pda: PublicKey;
  let vault1Pda: PublicKey;
  let authorityPda: PublicKey;
  let creatorToken0Ata: PublicKey;
  let creatorToken1Ata: PublicKey;
  let creatorLpAta: PublicKey;
  const configIndex = 2; // Different index from other tests

  before(async () => {
    // 1. Create config
    configPDA = get_amm_config_pda({
      index: configIndex,
      program_id: program.programId,
    });

    await program.methods
      .createConfig(
        configIndex,
        new anchor.BN(2500),
        new anchor.BN(0),
        new anchor.BN(100000),
        new anchor.BN(250000),
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

    // 3. Create ATAs and fund them with lots of tokens
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
    creatorToken0Ata = token0Account.address;
    creatorToken1Ata = token1Account.address;

    await mintTo(
      provider.connection,
      owner,
      token0Mint,
      creatorToken0Ata,
      owner,
      100_000_000_000
    ); // 100B
    await mintTo(
      provider.connection,
      owner,
      token1Mint,
      creatorToken1Ata,
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
    lpMintPda = get_lp_mint_pda({ program_id: program.programId, pool: poolPDA });
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
    creatorLpAta = getAssociatedTokenAddressSync(lpMintPda, owner.publicKey);

    // 5. Create the pool with initial liquidity
    await program.methods
      .createPool(
        configIndex,
        new anchor.BN(1_000_000_000),
        new anchor.BN(1_000_000_000),
        new anchor.BN(0)
      )
      .accounts({
        creator: owner.publicKey,
        ammConfig: configPDA,
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        creatorToken0: creatorToken0Ata,
        creatorToken1: creatorToken1Ata,
        feeReceiver: owner.publicKey,
      } as any)
      .signers([owner])
      .rpc();

    console.log("Pool ready for deposit test:", poolPDA.toBase58());
    
    // Wait for open_time to pass (pool sets open_time = block_timestamp + 1)
    console.log("Waiting for pool to open...");
    await new Promise(resolve => setTimeout(resolve, 2000));
  });

  it("deposits liquidity and receives LP tokens", async () => {
    // Get balances before deposit
    const lpBalanceBefore = await getAccount(provider.connection, creatorLpAta);
    const vault0Before = await getAccount(provider.connection, vault0Pda);
    const vault1Before = await getAccount(provider.connection, vault1Pda);

    const lpAmountToDeposit = new anchor.BN(500_000_000); // 0.5B LP tokens
    const maxToken0 = new anchor.BN(1_000_000_000); // Slippage tolerance
    const maxToken1 = new anchor.BN(1_000_000_000);

    // Execute deposit
    await program.methods
      .deposit(lpAmountToDeposit, maxToken0, maxToken1)
      .accounts({
        signer: owner.publicKey,
        poolState: poolPDA,
        ammConfig: configPDA,
        authority: authorityPda,
        token0Mint: token0Mint,
        token1Mint: token1Mint,
        signerToken0: creatorToken0Ata,
        signerToken1: creatorToken1Ata,
        token0Vault: vault0Pda,
        token1Vault: vault1Pda,
        lpMint: lpMintPda,
        signerLp: creatorLpAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      } as any)
      .signers([owner])
      .rpc();

    // Verify LP tokens received
    const lpBalanceAfter = await getAccount(provider.connection, creatorLpAta);
    const lpReceived = lpBalanceAfter.amount - lpBalanceBefore.amount;
    console.log("LP tokens received:", lpReceived.toString());
    assert(lpReceived > 0, "Should have received LP tokens");

    // Verify vault balances increased
    const vault0After = await getAccount(provider.connection, vault0Pda);
    const vault1After = await getAccount(provider.connection, vault1Pda);
    assert(
      vault0After.amount > vault0Before.amount,
      "Vault0 should have more tokens"
    );
    assert(
      vault1After.amount > vault1Before.amount,
      "Vault1 should have more tokens"
    );

    console.log("Deposit successful!");
    console.log("Vault0 increase:", (vault0After.amount - vault0Before.amount).toString());
    console.log("Vault1 increase:", (vault1After.amount - vault1Before.amount).toString());
  });
});
