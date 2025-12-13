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

describe("withdraw", () => {
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
  // Unique index per run (for devnet compatibility)
  const configIndex = (Math.floor(Date.now() / 1000) + 3000) % 65535;

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
    creatorToken0Ata = token0Account.address;
    creatorToken1Ata = token1Account.address;

    await mintTo(
      provider.connection,
      owner,
      token0Mint,
      creatorToken0Ata,
      owner,
      100_000_000_000
    );
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

    // 5. Create pool with initial liquidity
    await program.methods
      .createPool(
        configIndex,
        new anchor.BN(10_000_000_000), // 10B initial
        new anchor.BN(10_000_000_000),
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

    // Wait for pool to open
    await new Promise((resolve) => setTimeout(resolve, 2000));

    console.log("Pool ready for withdraw test:", poolPDA.toBase58());
  });

  it("withdraws liquidity and receives tokens back", async () => {
    // Get LP balance before
    const lpBalanceBefore = await getAccount(provider.connection, creatorLpAta);
    const userToken0Before = await getAccount(provider.connection, creatorToken0Ata);
    const userToken1Before = await getAccount(provider.connection, creatorToken1Ata);
    
    console.log("LP balance before:", lpBalanceBefore.amount.toString());
    
    // Withdraw half of LP tokens
    const lpToWithdraw = new anchor.BN(Number(lpBalanceBefore.amount) / 2);
    const minToken0 = new anchor.BN(1); // Minimum acceptable (for slippage)
    const minToken1 = new anchor.BN(1);

    await program.methods
      .withdraw(lpToWithdraw, minToken0, minToken1)
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

    // Verify LP tokens burned
    const lpBalanceAfter = await getAccount(provider.connection, creatorLpAta);
    const lpBurned = lpBalanceBefore.amount - lpBalanceAfter.amount;
    console.log("LP tokens burned:", lpBurned.toString());
    assert(lpBurned > 0, "LP tokens should have been burned");

    // Verify tokens received back
    const userToken0After = await getAccount(provider.connection, creatorToken0Ata);
    const userToken1After = await getAccount(provider.connection, creatorToken1Ata);
    
    const token0Received = userToken0After.amount - userToken0Before.amount;
    const token1Received = userToken1After.amount - userToken1Before.amount;
    
    console.log("Token0 received:", token0Received.toString());
    console.log("Token1 received:", token1Received.toString());
    
    assert(token0Received > 0, "Should have received token0");
    assert(token1Received > 0, "Should have received token1");

    console.log("Withdraw successful!");
  });
});
