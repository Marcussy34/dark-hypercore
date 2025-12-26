# Dark HyperCore L1

The "Darkpool-Native" Layer 1 Blockchain. > Combining the execution speed of Hyperliquid with the mathematical privacy of a Dark Pool.

## 📖 Introduction

Dark HyperCore is a high-frequency Layer 1 blockchain designed to resolve the Transparency Paradox in DeFi: the conflict between trustless settlement and trading privacy.

Standard blockchains (Ethereum, Solana) force institutional traders to leak their strategy to the public mempool, leading to MEV exploitation and front-running. Dark HyperCore leverages Trusted Execution Environments (TEEs) and a Proof of Trusted Execution (PoTE) consensus mechanism to enable:

*   **Sub-200ms Finality:** Matching centralized exchange (CEX) speeds.
*   **Encrypted State:** A "Black Box" order book where validators verify execution without seeing the trades.
*   **Neobank Integration:** A dual-layer architecture that pairs a private trading engine with a public, compliant banking layer.

## 🏰 Architecture: "The Dark Citadel"

The system operates on a novel Dual-Layer architecture running on a unified validator set.

### 1. Layer A: The "Lobby" (Public EVM)

*   **Role:** The Gateway & Neobank Interface.
*   **Tech Stack:** Forked Reth (Rust Ethereum) implementation.
*   **Function:** Handles user deposits, stablecoin on-ramps (USDC/USDT), and integrations with standard wallets (MetaMask, Rabby) and fintech providers (ZeroDev, Visa cards).
*   **Privacy Status:** Transparent. All inflows and outflows are visible, ensuring "Proof of Solvency" and easy auditing.

### 2. Layer B: The "Vault" (Dark Core)

*   **Role:** The High-Frequency Trading Engine.
*   **Tech Stack:** Native Rust bare-metal engine running inside Intel TDX Enclaves.
*   **Function:** Hosts the Central Limit Order Book (CLOB). Executes trades in complete privacy using fixed-point arithmetic.
*   **Privacy Status:** Opaque. Validators attest to the integrity of the code execution but cannot read the memory state.

### 3. The Native Bridge

*   **Mechanism:** An atomic memory-copy bridge between Layer A and Layer B.
*   **Speed:** Instant. Since both layers run on the same physical hardware (validators), there is no cross-chain latency.
*   **UX:** Users sign a "Session Key" on Layer A to authorize high-speed, pop-up-free trading on Layer B.

## 🛠️ Technical Stack

We strictly adhere to a Rust-Only monolithic architecture to ensure type safety and memory efficiency inside the TEE.

| Component | Technology | Reasoning |
| :--- | :--- | :--- |
| **Language** | Rust (Stable) | Memory safety without Garbage Collection (GC) pauses. |
| **TEE Hardware** | Intel TDX (Gen 4/5 Xeon) | Isolates the entire "Dark Kernel" VM, not just snippets (SGX). |
| **Consensus** | Pipelined HotStuff | "Jolteon" variant for <200ms optimistic responsiveness. |
| **Memory** | Slab Allocation | Pre-allocated memory to prevent "Secure Paging" performance cliffs. |
| **Math** | Fixed-Point (I80F48) | Strictly NO Floating Points to ensure deterministic consensus. |
| **Serialization** | SSZ (ssz_rs) | Standardized serialization for hashing and attestation. |

## 🧩 Core Concepts

### Proof of Trusted Execution (PoTE)

Unlike Proof of Work (hashing) or standard Proof of Stake (re-execution), our consensus relies on Hardware Attestations.

1.  **Execute:** The Leader executes a batch of trades inside their TEE.
2.  **Attest:** The TEE hardware signs a "Quote" proving the result is correct based on the open-source binary.
3.  **Verify:** Validators vote "Yes" if the Quote is valid. They do not need to see the transaction data to verify the block.

### View Keys (Auditing)

We implement a Dual-Key Architecture (similar to Monero/Secret Network):

*   **Spend Key:** Used to sign trades.
*   **View Key:** Used to decrypt trade history.
*   **Use Case:** A trader can share their View Key with an auditor or regulator to prove compliance without making their history public to the world.

## 🚀 Development Roadmap

### Phase 1: The Dark Kernel (Current Focus)

*   [ ] Build CLOB with Slab allocation.
*   [ ] Implement deterministic matching engine (Fixed-Point).
*   [ ] Benchmark 1M TPS in local userspace.

### Phase 2: Consensus & Networking

*   [ ] Implement Pipelined HotStuff (HyperBFT equivalent).
*   [ ] Integrate intel-dcap-rs for dummy attestation generation.
*   [ ] Build Encrypted Gossip Layer (Mempool).

### Phase 3: The "Citadel" Integration

*   [ ] Fork Reth for Layer A (EVM).
*   [ ] Build Shared Memory Bridge between EVM and Dark Core.
*   [ ] Implement Session Key logic (EIP-4337 compatibility).

### Phase 4: Mainnet Launch

*   [ ] Heterogeneous Hardware Support (AMD SEV-SNP integration).
*   [ ] Futarchy Governance Module.
*   [ ] Genesis Block & Token Generation Event (TGE).

## ⚡ Getting Started

### Prerequisites

*   **Rust:** 1.75+ (Nightly required for some benchmarks).
*   **Hardware:** Local development works on Mac/Linux. Production requires Intel Ice Lake (or newer) with SGX/TDX enabled.

### Installation

```bash
# Clone the repository
git clone https://github.com/your-org/dark-hypercore.git
cd dark-hypercore

# Build the Kernel (Phase 1)
cargo build --release --package engine

# Run the Stress Test
cargo bench --bench engine_stress_test
```

## 🤝 Contributing

We are an "Experimental Labs" business. We prioritize Innovation over "Number Go Up."

*   **Strict No-Go:** No generic DeFi forks, no rehypothecated collateral tokens.
*   **Code Style:** `rustfmt` is mandatory. No `unsafe` blocks allowed inside the TEE modules without TSC (Technical Steering Committee) approval.

## 📜 License

This project is licensed under the MIT License - see the LICENSE file for details.

Built with silence in the Dark Citadel.
