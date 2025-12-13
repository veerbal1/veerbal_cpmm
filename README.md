# Veerbal CPMM - Constant Product Market Maker on Solana

A production-grade Constant Product Market Maker (CPMM) implementation on Solana, following Raydium's architecture. Built with Anchor framework.

## 🚀 Live on Devnet

| Detail | Value |
|--------|-------|
| **Program ID** | `C6TCz92bpYjWgty9mwrAoNh7u6RSdmyBRB4dMoBGgMrA` |
| **Network** | Solana Devnet |
| **Status** | ✅ Deployed & Tested |

[View on Solscan](https://solscan.io/account/C6TCz92bpYjWgty9mwrAoNh7u6RSdmyBRB4dMoBGgMrA?cluster=devnet)

## ✨ Features

- **Constant Product AMM** - x * y = k invariant
- **Multi-tier Fee System** - Trade, protocol, fund, and creator fees
- **Dual Swap Modes** - Base input (exact input) and base output (exact output)
- **Full Liquidity Management** - Deposit, withdraw with slippage protection
- **Fee Collection** - Separate collection for protocol, fund, and creator fees
- **Production Security** - PDA validation, checked arithmetic, owner checks

## 🏗️ Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         AMM Config                               │
│  (Fee rates, protocol/fund owners, pool creation settings)      │
└─────────────────────────────────────────────────────────────────┘
                                 │
                                 ▼
┌─────────────────────────────────────────────────────────────────┐
│                          Pool State                              │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐             │
│  │ Token 0     │  │ Token 1     │  │ LP Mint     │             │
│  │ Vault       │  │ Vault       │  │             │             │
│  └─────────────┘  └─────────────┘  └─────────────┘             │
│                                                                  │
│  Fee Accumulators: protocol_fee, fund_fee, creator_fee          │
└─────────────────────────────────────────────────────────────────┘
```

## 📦 Instructions

| Instruction | Description |
|-------------|-------------|
| `create_config` | Create AMM configuration with fee rates |
| `create_pool` | Initialize a new liquidity pool |
| `deposit` | Add liquidity, receive LP tokens |
| `withdraw` | Remove liquidity, burn LP tokens |
| `swap` | Swap with exact input amount |
| `swap_base_output` | Swap for exact output amount |
| `collect_protocol_fee` | Collect accumulated protocol fees |
| `collect_fund_fee` | Collect accumulated fund fees |
| `collect_creator_fee` | Collect accumulated creator fees |

## 🔐 Security Features

- ✅ **PDA Validation** - All derived accounts verified with seeds
- ✅ **Checked Arithmetic** - No overflow/underflow possible
- ✅ **Owner Checks** - Admin, protocol, fund, creator permissions
- ✅ **Slippage Protection** - Min/max amount enforcement
- ✅ **K Invariant** - Constant product verified on every swap
- ✅ **Open Time Gating** - Pools can have delayed activation

## 🧪 Testing

```bash
# Run all tests on localnet
anchor test

# Run tests on devnet (after deployment)
anchor test --skip-local-validator --skip-deploy
```

### Test Coverage

| Test Suite | Tests | Status |
|------------|-------|--------|
| create-config | 1 | ✅ |
| create-pool | 1 | ✅ |
| deposit | 1 | ✅ |
| withdraw | 1 | ✅ |
| swap (base input) | 1 | ✅ |
| swap (base output) | 1 | ✅ |
| collect-fees | 3 | ✅ |
| **Total** | **9** | **✅ All Passing** |

## 🛠️ Development Setup

### Prerequisites

- Rust 1.70+
- Solana CLI 1.17+
- Anchor 0.30+
- Node.js 18+

### Build

```bash
anchor build
```

### Deploy to Devnet

```bash
# Configure for devnet
solana config set --url devnet

# Get devnet SOL
solana airdrop 2

# Deploy
anchor deploy
```

## 📁 Project Structure

```
programs/veerbal_cpmm/src/
├── lib.rs                 # Program entrypoint
├── constants.rs           # PDA seeds
├── error.rs              # Custom errors
├── states/
│   ├── config.rs         # AmmConfig account
│   └── pool.rs           # PoolState account
├── curve/
│   ├── constant_product.rs  # x*y=k math
│   └── fees.rs           # Fee calculations
└── instructions/
    ├── create_config.rs
    ├── initialize.rs     # create_pool
    ├── deposit.rs
    ├── withdraw.rs
    ├── swap_base_input.rs
    ├── swap_base_output.rs
    ├── collect_creator_fee.rs
    ├── collect_protocol_fee.rs
    └── collect_fund_fee.rs
```

## 📊 Fee Structure

Fees are calculated as parts per million (1,000,000 = 100%):

| Fee Type | Description |
|----------|-------------|
| `trade_fee_rate` | Total fee taken from swaps |
| `protocol_fee_rate` | Portion of trade fee to protocol |
| `fund_fee_rate` | Portion of trade fee to fund |
| `creator_fee_rate` | Portion of trade fee to pool creator |

## 🙏 Acknowledgments

This implementation follows [Raydium's CPMM](https://github.com/raydium-io/raydium-cp-swap) architecture as a learning exercise.

## 📄 License

MIT
