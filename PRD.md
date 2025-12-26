# Product Requirement Document: Dark HyperCore

**Project Name:** Dark HyperCore  
**Status:** Phase 1 (Development - Local Kernel)  
**Last Updated:** December 26, 2025  
**Role:** Senior Rust Protocol Engineer

---

## 1. Executive Summary

Dark HyperCore is a high-frequency Layer 1 blockchain that resolves the **Transparency Paradox** in DeFi. By combining hardware-based isolation (Intel TDX) with a bare-metal, monolithic Rust engine, we enable institutional-grade privacy with sub-second finality.

**Vision:** Build a high-performance Layer 1 blockchain that combines Hyperliquid-level execution speed with TEE-native privacy to create a "Darkpool by Default" financial primitive.

---

## 2. Target Audience

### Institutional Market Makers
Entities requiring shield-trading strategies and private balance sheets.

### High-Frequency Traders (HFT)
Users needing sub-200ms finality without the risk of mempool exploitation.

### Privacy-Conscious Individuals
Retail users who want CEX-level speed with DEX-level self-custody and privacy.

---

## 3. Core Architecture

The system is built on a **"Privacy-First"** stack:

### Execution (The Dark Kernel)
A native Rust Central Limit Order Book (CLOB) optimized with Slab allocation. It resides in the TEE to ensure all order data and matching logic are hidden from the host OS.

### Privacy (TEE-Native)
All sensitive computation runs within hardware-attested Intel TDX or AWS Nitro enclaves to ensure data is encrypted even "in-use".

### Consensus: Proof of Trusted Execution (PoTE)
A consensus mechanism where validators verify hardware-generated Remote Attestation Quotes instead of re-executing private transactions. Implements Pipelined HotStuff (Jolteon variant) where:
- Proposal View `v` extends `QC_{v-1}`
- Voting logic replaces standard "Digital Signature Verification" with "TEE Attestation Verification" (PoTE)

### Auditability (View Keys)
A dual-key system (Spend Key vs. View Key) for selective disclosure and regulatory compliance.

---

## 4. Technical Constraints (Strict Adherence)

### 4.1 Language & Architecture
- **Language:** Stable Rust only
- **Architecture:** Monolithic binary to simplify TEE attestation and state management
- **No Async in Hot Path:** The matching engine must remain synchronous and deterministic to ensure maximum throughput

### 4.2 Data & Math
- **Serialization:** `ssz_rs` (Simple Serialize) for all consensus and state data
- **Math:** `rust_decimal` or custom `I80F48` fixed-point math. Floating points are strictly prohibited
- **Memory:** Use the `slab` crate for pre-allocated order storage to prevent heap thrashing and page faults inside the TEE

### 4.3 Performance Targets

| Metric | Target |
|--------|--------|
| **Throughput** | >100,000 orders per second per shard |
| **Latency** | <200ms end-to-end (Signature to Finality) |
| **Execution** | Deterministic (fixed-point math, no floating points) |

### 4.4 Hardware Constraints

**Validation Environment:**
- Intel Xeon Scalable 4th Gen (Sapphire Rapids) or 5th Gen (Emerald Rapids)
- Validators must support TDX Module v1.5+ to handle "Secure Paging" efficiently if the Order Book exceeds EPC limits

**Memory Management:**
- Slab allocation for O(1) memory access to prevent TEE performance degradation

### 4.5 External Interfaces (The "Handshake" API)

**Input Standard:**
- The Engine accepts `EncryptedBlobs` containing SSZ-formatted orders

**Bridge Port:**
- A dedicated memory region (Shared Memory Ring Buffer) reserved for the external Neobank Layer to push "Deposit/Withdrawal" events

**Output Stream:**
- A read-only stream publishing State Roots and Attestation Quotes (but not trade data) every 200ms for external verification

---

## 5. User Stories

### As a trader
I want to place a limit order that is hidden from the public mempool so that I am not front-run by MEV bots.

### As an auditor
I want to view a specific user's trade history via their View Key to ensure regulatory compliance.

### As a developer
I want to build private automated strategies that sign trades using local session keys for speed.

---

## 6. Success Metrics

### Latency Benchmarks
Consistently achieving sub-300ms round-trip times in TDX-simulated environments.

### Privacy Integrity
Successful generation and verification of Remote Attestation (Quotes) for every batch of trades.

---

## 7. Risks and Dependencies

### Hardware Supply
Dependence on Intel/AMD for secure silicon availability.

### Side-Channel Attacks
Risks of TEE-specific vulnerabilities (e.g., cache timing attacks).

### Regulatory Landscape
Potential friction with global AML/KYC requirements for private ledgers.

---

## 8. Development Roadmap

### Phase 1: The Dark Kernel *(Current Focus)*

**Goal:** Build the single-threaded deterministic matching engine.

**Execution Environment:**
- Build and benchmark in **standard userspace (User Mode)** first, NOT inside the TEE yet
- **Why:** Establish a raw performance baseline (e.g., "150k TPS on raw CPU")
- This baseline is critical for measuring TEE overhead when moving to Phase 2
- You need to know exactly how much performance the TEE is "costing" you

**Deterministic Order Matching:**
- Implement a `CLOB` struct using a Limit Order Book model
- Use `Slab` to pre-allocate memory slots for `OrderNode` structs
- Implement `match_orders` using fixed-point arithmetic

**SSZ Data Structures:**
- Define `Order`, `Trade`, and `ExecutionReceipt` structs deriving `ssz_rs::Serialize`

**Local Stress Test:**
- Benchmark 1M random orders in a tight loop to ensure sub-millisecond execution
- Record baseline TPS metrics for future TEE comparison

### Phase 2: Consensus Layer (PoTE)
- **Migrate Phase 1 engine into TEE environment** (Intel TDX/AWS Nitro)
- Measure and document TEE performance overhead vs. Phase 1 baseline
- Implement the Jolteon pipelined consensus engine
- Integrate `intel-dcap-rs` for handling hardware quotes (mocked for local dev)

### Phase 3: Privacy & UX SDK
- Implement View Key derivation and client-side (WASM) decryption
- Establish Session Keys for automated high-frequency trading

### Phase 4: Mainnet & Governance
- Launch Heterogeneous hardware support (Intel + AMD)
- Deploy Futarchy-based governance for protocol upgrades

---

## 9. Technical Stack

- **Language:** Stable Rust (bare-metal, no_std where applicable)
- **Architecture:** Monolithic binary
- **TEE:** Intel TDX / AWS Nitro Enclaves
- **Consensus:** Pipelined HotStuff (Jolteon variant) with PoTE
- **Serialization:** `ssz_rs` (Simple Serialize)
- **Math:** `rust_decimal` or custom `I80F48` fixed-point (no floating point)
- **Memory:** `slab` crate for pre-allocated order storage
- **Execution Model:** Synchronous, deterministic (no async in hot path)

---

## 10. Implementation Requirements

### Phase 1 Deliverables

**Core Data Structures:**
- `Order` struct with SSZ serialization
- `Trade` struct with SSZ serialization
- `ExecutionReceipt` struct with SSZ serialization
- `CLOB` struct using Limit Order Book model
- `OrderNode` structs allocated via `Slab`

**Matching Engine:**
- Single-threaded, synchronous `match_orders` function
- Fixed-point arithmetic for all price/quantity calculations
- Deterministic execution guarantees

**Testing & Benchmarking:**
- Local stress test: 1M random orders
- Target: Sub-millisecond execution per order
- Verify deterministic state roots across runs

### Input/Output Specifications

**Input Standard:**
- Accept `EncryptedBlobs` containing SSZ-formatted orders

**Output Stream:**
- Publish State Roots every 200ms
- Publish Attestation Quotes every 200ms
- No trade data exposed in output stream

## 11. Next Steps

1. Set up Rust project with required dependencies (`ssz_rs`, `slab`, `rust_decimal`)
2. Implement core CLOB data structures with SSZ serialization
3. Build synchronous matching engine with fixed-point math
4. Create benchmarking harness for 1M order stress test
5. Verify deterministic execution and sub-millisecond performance

---

*This is a living document and will be updated as the project evolves.*

