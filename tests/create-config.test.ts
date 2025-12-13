import * as anchor from "@coral-xyz/anchor";
import { VeerbalCpmm } from "../target/types/veerbal_cpmm";
import { get_amm_config_pda } from "./utils";
import { assert } from "chai";

describe("Create AMM config", () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const provider = anchor.getProvider();

  const program = anchor.workspace.VeerbalCpmm as anchor.Program<VeerbalCpmm>;
  const owner = anchor.Wallet.local().payer;

  it("creates config with valid fee rates", async () => {
    const index = 0;
    const configPda = get_amm_config_pda({
      index,
      program_id: program.programId,
    });
    await program.methods
      .createConfig(
        index, // index: u16
        new anchor.BN(2500), // trade_fee_rate: 0.25%
        new anchor.BN(0), // creator_fee_rate
        new anchor.BN(100000), // protocol_fee_rate: 10%
        new anchor.BN(250000), // fund_fee_rate: 25%
        new anchor.BN(1000000000) // create_pool_fee: 1 SOL
      )
      .accounts({
        owner: owner.publicKey,
        ammConfig: configPda,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .signers([owner])
      .rpc();

    const config = await program.account.ammConfig.fetch(configPda);
    assert.equal(config.tradeFeeRate.toNumber(), 2500);
    assert.equal(config.index, index);
  });
});
