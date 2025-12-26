# Product Requirement Document: Dark HyperCore

**Project Name:** Dark HyperCore  
**Status:** Phase 1 (Development - Local Kernel)  
**Last Updated:** December 26, 2025

---

## 1. Executive Summary

Dark HyperCore addresses the **Transparency Paradox** in DeFi: while public ledgers provide trustless settlement, they expose institutional traders to MEV, front-running, and copy-trading. 

By leveraging hardware-based isolation (Intel TDX) and a bare-metal Rust engine, Dark HyperCore enables sub-second, private high-frequency trading on-chain.

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

### Execution (Dark Engine)
A native Rust Central Limit Order Book (CLOB) optimized with Slab allocation to minimize memory paging inside TEEs.

### Privacy (TEE-Native)
Operations run within hardware-attested Intel TDX or AWS Nitro enclaves to ensure data is encrypted even "in-use".

### Consensus (PoTE)
A "Proof of Trusted Execution" (PoTE) protocol utilizing Attestation-Based Voting on top of a pipelined BFT algorithm (HotStuff/HyperBFT). Nodes verify hardware signatures rather than re-executing private transactions.

### Auditability (View Keys)
A dual-key system (Spend Key vs. View Key) allowing users to selectively disclose their transaction history for compliance without revealing it to the public.

---

## 4. Technical Requirements

### 4.1 Performance Targets

| Metric | Target |
|--------|--------|
| **Throughput** | >100,000 orders per second per shard |
| **Latency** | <200ms end-to-end (Signature to Finality) |
| **Execution** | Deterministic (fixed-point math, no floating points) |

### 4.2 Hardware Constraints

**Validation Environment:**
- Intel Xeon Scalable 4th Gen (Sapphire Rapids) or 5th Gen (Emerald Rapids)
- Validators must support TDX Module v1.5+ to handle "Secure Paging" efficiently if the Order Book exceeds EPC limits

**Memory Management:**
- Slab allocation for O(1) memory access to prevent TEE performance degradation

### 4.3 External Interfaces (The "Handshake" API)

**Input Standard:**
- The Engine accepts `EncryptedBlobs` containing standardized S.S.Z (Simple Serialize) formatted orders

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

### Phase 1: Dark Kernel *(Current)*
Native Rust Order Book with Slab Allocation.

### Phase 2: Consensus Layer
Implementation of PoTE and HyperBFT.

### Phase 3: Privacy SDK
View keys and encrypted log explorer.

### Phase 4: Mainnet & Governance
Heterogeneous hardware set and Futarchy.

---

## 9. Technical Stack

- **Language:** Rust (bare-metal, no_std where applicable)
- **TEE:** Intel TDX / AWS Nitro Enclaves
- **Consensus:** HotStuff/HyperBFT with PoTE
- **Serialization:** Simple Serialize (SSZ)
- **Memory:** Slab allocation for deterministic performance

---

## 10. Next Steps

1. Complete Phase 1: Implement core CLOB engine with slab allocator
2. Establish benchmarking framework for latency/throughput testing
3. Research TDX integration requirements and SDK options
4. Design cryptographic primitives for dual-key system
5. Create initial test suite for order matching logic

---

*This is a living document and will be updated as the project evolves.*

