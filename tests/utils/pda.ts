import { PublicKey } from "@solana/web3.js";

export const get_amm_config_pda = ({
  index,
  program_id,
}: {
  index: number;
  program_id: PublicKey;
}): PublicKey => {
  const index_buffer = Buffer.alloc(2);
  index_buffer.writeUint16BE(index);

  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("AMM_CONFIG"), index_buffer],
    program_id
  );

  return pda;
};

export const get_pool_pda = ({
  program_id,
  config_pda,
  mint0,
  mint1,
}: {
  program_id: PublicKey;
  config_pda: PublicKey;
  mint0: PublicKey;
  mint1: PublicKey;
}): PublicKey => {
  const [pda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from("POOL"),
      config_pda.toBuffer(),
      mint0.toBuffer(),
      mint1.toBuffer(),
    ],
    program_id
  );

  return pda;
};

export const get_vault_pda = ({
  program_id,
  pool,
  mint,
}: {
  program_id: PublicKey;
  pool: PublicKey;
  mint: PublicKey;
}) => {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("VAULT_SEED"), pool.toBuffer(), mint.toBuffer()],
    program_id
  );

  return pda;
};

export const get_lp_mint_pda = ({
  program_id,
  pool,
}: {
  program_id: PublicKey;
  pool: PublicKey;
}) => {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("LP_MINT"), pool.toBuffer()],
    program_id
  );

  return pda;
};

export const get_authority_pda = ({
  program_id,
}: {
  program_id: PublicKey;
}) => {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from("AUTH_SEED")],
    program_id
  );

  return pda;
};

export const orderMints = (
  mintA: PublicKey,
  mintB: PublicKey
): [PublicKey, PublicKey] => {
  return mintA.toBuffer().compare(mintB.toBuffer()) < 0
    ? [mintA, mintB]
    : [mintB, mintA];
};
