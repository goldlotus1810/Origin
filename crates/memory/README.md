# memory

> DreamCycle (offline consolidation: STM -> cluster -> LCA -> QR proposals) and AAM (stateless approve/reject orchestrator).

## Dependencies
- ucd
- olang
- silk
- context
- agents
- libm

## Files
| File | Purpose |
|------|---------|
| dream.rs | DreamCycle: scan STM top-N, dual-threshold clustering (0.3*chain_sim + 0.4*hebbian_weight + 0.3*co_act_ratio), LCA -> DreamProposal -> AAM review |
| proposal.rs | ProposalKind (NewNode/PromoteQR/NewEdge/SupersedeQR), DreamProposal, AAM (stateless reviewer), AAMDecision (Approved/Rejected/Pending) |

## Key API
```rust
pub fn DreamCycle::run(&self, stm: &ShortTermMemory, graph: &SilkGraph, ts: i64) -> DreamResult
pub fn DreamProposal::new_node(chain: MolecularChain, emotion: EmotionTag, sources: Vec<u64>, confidence: f32, ts: i64) -> Self
pub fn DreamProposal::promote_qr(chain_hash: u64, fire_count: u32, confidence: f32, ts: i64) -> Self
pub fn AAM::review(&self, proposal: &DreamProposal) -> AAMDecision
pub fn AAM::review_batch<'a>(&self, proposals: &'a [DreamProposal]) -> Vec<&'a DreamProposal>
```

## Rules
- 15: Only 2 Agents (AAM + LeoAI) -- do not add more.
- 10: Append-only -- NO DELETE, NO OVERWRITE.
- 17: Fibonacci threshold: promote when fire_count >= Fib[depth].
- Cluster threshold >= 0.6 by default.
- AAM requires confidence >= 0.5, sources >= 3 for NewNode, fire_count >= 5 for PromoteQR, confidence >= 0.8 for SupersedeQR.

## Test
```bash
cargo test -p memory
```
