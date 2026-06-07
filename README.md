# Provenance Chain

[![crates.io](https://img.shields.io/crates/v/provenance-chain.svg)](https://crates.io/crates/provenance-chain)
[![docs.rs](https://docs.rs/provenance-chain/badge.svg)](https://docs.rs/provenance-chain)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> **Knowledge provenance tracking with trust scores, source reliability, and full lineage tracing.**

---

## The Problem

When agents share knowledge, the question isn't just "what do we know?" but "where did this knowledge come from, and how much should we trust it?" Without provenance tracking, knowledge propagates blindly — a low-confidence rumor carries the same weight as a verified observation.

## Why This Exists

Provenance Chain tracks every derivation step in a knowledge chain: who created it, when, from what source, and with what confidence. Combined with source reliability tracking over time, it computes **trust scores** that weight knowledge by its lineage quality. Longer chains introduce more uncertainty; unreliable sources drag down trust.

## Architecture

```
  ┌──────────┐    ┌──────────┐    ┌──────────┐
  │ Sensor   │───→│ Agent A  │───→│ Agent B  │───→ Decision
  │ (orig)   │    │ (derive) │    │ (derive) │
  │ conf: 0.9│    │ conf: 0.8│    │ conf: 0.7│
  └──────────┘    └──────────┘    └──────────┘
       │               │               │
       └───────────────┼───────────────┘
                       │
                ┌──────▼──────┐
                │   Trust     │
                │  Scoring    │
                │             │
                │ avg_conf    │
                │ × depth     │
                │ × source    │
                │ = TRUST     │
                └─────────────┘
```

## Installation

```toml
[dependencies]
provenance-chain = "0.1"
```

## API Reference

### `ProvenanceNode`

A node tracking the origin of a piece of knowledge:

```rust
use provenance_chain::ProvenanceNode;

let node = ProvenanceNode::new("n1", "agent_a", 1.0, "sensor", 0.9);
assert_eq!(node.who, "agent_a");
assert_eq!(node.confidence, 0.9);
```

### `ProvenanceChain`

A chain of provenance linking knowledge to its origin:

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
chain.append(ProvenanceNode::new("1", "agent_a", 1.0, "sensor", 0.9));
chain.append(ProvenanceNode::new("2", "agent_b", 2.0, "derived", 0.8));

assert_eq!(chain.depth(), 2);
assert_eq!(chain.origin().unwrap().who, "agent_a");
assert_eq!(chain.latest().unwrap().who, "agent_b");
```

### `TrustScore` & `compute_trust`

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
chain.append(ProvenanceNode::new("1", "a", 1.0, "sensor", 0.9));

let mut reliability = SourceReliability::new();
reliability.record("sensor", 0.9);

let trust = compute_trust(&chain, &reliability);
assert!(trust.is_trusted()); // ≥ 0.7
```

### `SourceReliability`

Track source accuracy over time:

```rust
use provenance_chain::SourceReliability;

let mut rel = SourceReliability::new();
rel.record("api", 0.8);
rel.record("api", 1.0);
assert!((rel.get("api") - 0.9).abs() < 0.001);
assert_eq!(rel.observations("api"), 2);
assert_eq!(rel.get("unknown"), 0.5); // default
```

### `ProvenanceQuery`

Query interface for tracing origins:

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
chain.append(ProvenanceNode::new("1", "alice", 1.0, "sensor", 0.9));
chain.append(ProvenanceNode::new("2", "bob", 2.0, "api", 0.8));
chain.append(ProvenanceNode::new("3", "carol", 3.0, "sensor", 0.7));

let query = ProvenanceQuery::new(&chain);
let sensors = query.from_source("sensor");
assert_eq!(sensors.len(), 2);

let lineage = query.lineage();
// [("alice", "sensor"), ("bob", "api"), ("carol", "sensor")]
```

## Usage Examples

### Example 1: Track Knowledge Derivation

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
chain.append(ProvenanceNode::new("obs", "sensor-1", 100.0, "temperature", 0.95));
chain.append(ProvenanceNode::new("inf", "agent-x", 101.0, "derived", 0.8));
chain.append(ProvenanceNode::new("dec", "agent-y", 102.0, "derived", 0.7));

let mut rel = SourceReliability::new();
rel.record("temperature", 0.95);
rel.record("derived", 0.75);

let trust = compute_trust(&chain, &rel);
println!("Knowledge trust: {:.2} ({})", trust.0,
    if trust.is_trusted() { "TRUSTED" } else { "UNCERTAIN" });
```

### Example 2: Query by Time and Agent

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
chain.append(ProvenanceNode::new("1", "alice", 5.0, "s", 0.9));
chain.append(ProvenanceNode::new("2", "bob", 15.0, "s", 0.8));
chain.append(ProvenanceNode::new("3", "alice", 25.0, "s", 0.7));

let query = ProvenanceQuery::new(&chain);
assert_eq!(query.by_agent("alice").len(), 2);
assert_eq!(query.in_time_range(10.0, 20.0).len(), 1);
```

### Example 3: Deep Chain Trust Decay

```rust
use provenance_chain::*;

let mut chain = ProvenanceChain::new();
for i in 0..10 {
    chain.append(ProvenanceNode::new(
        format!("n{}", i), "agent", i as f64, "derived", 0.8
    ));
}

let rel = SourceReliability::new();
let trust = compute_trust(&chain, &rel);
// Deep chains have lower trust due to depth penalty
// depth_factor = 1 / (1 + 0.1 * (10 - 1)) = 0.526
```

## Trust Computation Formula

```
trust = avg_confidence × depth_factor × avg_source_reliability

depth_factor = 1 / (1 + 0.1 × (depth - 1))
```

- **avg_confidence**: Mean confidence across all chain nodes
- **depth_factor**: Penalizes longer derivation chains
- **avg_source_reliability**: Mean reliability of all sources in chain

## Performance

| Operation | Complexity |
|-----------|-----------|
| Chain append/prepend | O(1) |
| Compute trust | O(n) |
| Source reliability update | O(1) |
| Query by source/agent | O(n) |
| Lineage trace | O(n) |

## License

Licensed under the [MIT License](LICENSE).

## Contributing

1. Fork the repository
2. Create a feature branch
3. Write tests
4. Push and open a Pull Request
