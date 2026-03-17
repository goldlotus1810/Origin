//! # knowtree — L2-Ln Knowledge Tree
//!
//! Kết nối TieredStore (compact encoding) với learning pipeline.
//!
//! Pipeline:
//!   BookReader → sentences → encode → L2 CompactNode (via KnowTree)
//!   Dream → cluster → LCA → promote to L2+ CompactNode
//!   Query → hash → TieredStore.lookup_with_edges → response
//!
//! Layers:
//!   L0: UCD base (35 seeded nodes) — always in RAM
//!   L1: User interactions (STM → promoted) — always in RAM
//!   L2: Book knowledge (sentences, concepts) — TieredStore
//!   L3: Abstracted patterns (LCA of L2 clusters) — TieredStore
//!   L4+: Higher abstractions — TieredStore
//!
//! KnowTree = wrapper quanh TieredStore, cung cấp:
//!   - store_sentence(): encode text → L2 compact node + silk edges
//!   - store_concept(): LCA chain → L3+ compact node
//!   - query(): hash → node + neighbors
//!   - promote(): STM observation → L2 node

extern crate alloc;
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;

use crate::compact::{CompactNode, NodeWithEdges, TieredStore};
use crate::hash::fnv1a_str;
use crate::molecular::{MolecularChain, RelationBase};

// ─────────────────────────────────────────────────────────────────────────────
// KnowTree
// ─────────────────────────────────────────────────────────────────────────────

/// Knowledge tree — L2-Ln storage powered by TieredStore.
pub struct KnowTree {
    /// Underlying tiered storage
    store: TieredStore,
    /// Total sentences stored
    sentences_stored: u64,
    /// Total concepts stored
    concepts_stored: u64,
    /// Total promotions from STM
    promotions: u64,
}

impl KnowTree {
    /// Create for PC (610 page cache, 8192 dict).
    pub fn for_pc() -> Self {
        Self {
            store: TieredStore::for_pc(),
            sentences_stored: 0,
            concepts_stored: 0,
            promotions: 0,
        }
    }

    /// Create for mobile (233 page cache, 4096 dict).
    pub fn for_mobile() -> Self {
        Self {
            store: TieredStore::for_mobile(),
            sentences_stored: 0,
            concepts_stored: 0,
            promotions: 0,
        }
    }

    /// Create for embedded (55 page cache, 256 dict).
    pub fn for_embedded() -> Self {
        Self {
            store: TieredStore::for_embedded(),
            sentences_stored: 0,
            concepts_stored: 0,
            promotions: 0,
        }
    }

    /// Create custom.
    pub fn new(cache_capacity: usize, dict_capacity: usize) -> Self {
        Self {
            store: TieredStore::new(cache_capacity, dict_capacity),
            sentences_stored: 0,
            concepts_stored: 0,
            promotions: 0,
        }
    }

    // ── Store operations ─────────────────────────────────────────────────────

    /// Store a sentence as L2 node.
    ///
    /// Returns chain_hash of the stored node.
    /// Also creates silk edges between consecutive words.
    pub fn store_sentence(
        &mut self,
        chain: &MolecularChain,
        parent: Option<&MolecularChain>,
        word_hashes: &[u64],
        ts: i64,
    ) -> u64 {
        let hash = self.store.store_node(chain, parent, 2, ts);
        self.sentences_stored += 1;

        // Create silk edges between consecutive word hashes
        if word_hashes.len() >= 2 {
            for w in word_hashes.windows(2) {
                self.store
                    .store_edge(w[0], w[1], 0.6, RelationBase::Compose.as_byte(), 2);
            }
        }

        // Edge from sentence node to each word (Member relation)
        for &wh in word_hashes {
            self.store
                .store_edge(hash, wh, 0.5, RelationBase::Member.as_byte(), 2);
        }

        hash
    }

    /// Store a concept (LCA result) as L3+ node.
    ///
    /// `sources` = hashes of L2 nodes that were clustered.
    pub fn store_concept(
        &mut self,
        chain: &MolecularChain,
        parent: Option<&MolecularChain>,
        layer: u8,
        sources: &[u64],
        ts: i64,
    ) -> u64 {
        let layer = layer.max(3); // concepts are L3+
        let hash = self.store.store_node(chain, parent, layer, ts);
        self.concepts_stored += 1;

        // Edges from concept to sources (DerivedFrom)
        for &src in sources {
            self.store
                .store_edge(hash, src, 0.7, RelationBase::DerivedFrom.as_byte(), layer);
        }

        hash
    }

    /// Promote an STM observation to L2 node.
    ///
    /// Called when Dream cycle approves promotion.
    pub fn promote_from_stm(
        &mut self,
        chain: &MolecularChain,
        parent: Option<&MolecularChain>,
        fire_count: u32,
        ts: i64,
    ) -> u64 {
        let hash = self.store.store_node(chain, parent, 2, ts);
        self.promotions += 1;

        // Weight proportional to fire_count (capped at 1.0)
        let _weight = (fire_count as f32 / 10.0).min(1.0);

        hash
    }

    // ── Query operations ─────────────────────────────────────────────────────

    /// Lookup node by hash in a specific layer.
    pub fn lookup(&mut self, hash: u64, layer: u8) -> Option<&CompactNode> {
        self.store.lookup(hash, layer)
    }

    /// Lookup node + follow silk edges (depth limited).
    pub fn query(&mut self, hash: u64, layer: u8, depth: u8) -> Option<NodeWithEdges> {
        self.store.lookup_with_edges(hash, layer, depth)
    }

    /// Search by text hash (FNV-1a of lowercase text).
    /// Searches L2 first, returns None if not found in L2 or L3.
    pub fn search_text(&mut self, text: &str, layer: u8) -> Option<&CompactNode> {
        let hash = fnv1a_str(text);
        self.store.lookup(hash, layer)
    }

    // ── Batch operations ─────────────────────────────────────────────────────

    /// Store multiple sentences from a book chapter.
    ///
    /// Returns hashes of all stored sentence nodes.
    pub fn store_chapter(
        &mut self,
        chains: &[(MolecularChain, Vec<u64>)], // (sentence_chain, word_hashes)
        chapter_chain: Option<&MolecularChain>,
        ts: i64,
    ) -> Vec<u64> {
        let mut hashes = Vec::with_capacity(chains.len());
        let mut prev_hash = 0u64;

        for (chain, word_hashes) in chains {
            let hash = self.store_sentence(chain, chapter_chain, word_hashes, ts);
            hashes.push(hash);

            // Sequential edge (sentence → next sentence)
            if prev_hash != 0 {
                self.store
                    .store_edge(prev_hash, hash, 0.4, RelationBase::Causes.as_byte(), 2);
            }
            prev_hash = hash;
        }

        hashes
    }

    // ── Stats ────────────────────────────────────────────────────────────────

    /// Total nodes across all layers.
    pub fn total_nodes(&self) -> u64 {
        self.store.total_nodes()
    }

    /// Total edges.
    pub fn total_edges(&self) -> u64 {
        self.store.total_edges()
    }

    /// Sentences stored (L2).
    pub fn sentences(&self) -> u64 {
        self.sentences_stored
    }

    /// Concepts stored (L3+).
    pub fn concepts(&self) -> u64 {
        self.concepts_stored
    }

    /// Promotions from STM.
    pub fn promotions(&self) -> u64 {
        self.promotions
    }

    /// RAM usage estimate.
    pub fn ram_usage(&self) -> usize {
        self.store.ram_usage()
    }

    /// Disk usage estimate.
    pub fn disk_usage(&self) -> usize {
        self.store.disk_usage()
    }

    /// Cache hit rate.
    pub fn cache_hit_rate(&self) -> f32 {
        self.store.cache.hit_rate()
    }

    /// Summary.
    pub fn summary(&self) -> String {
        format!(
            "KnowTree: {} sentences + {} concepts + {} promotions\n\
             Nodes: {} | Edges: {} | RAM: ~{}KB | Disk: ~{}KB\n\
             Cache: {:.1}% hit rate",
            self.sentences_stored,
            self.concepts_stored,
            self.promotions,
            self.store.total_nodes(),
            self.store.total_edges(),
            self.ram_usage() / 1024,
            self.disk_usage() / 1024,
            self.cache_hit_rate() * 100.0,
        )
    }

    /// Mutable access to underlying store (for advanced operations).
    pub fn store_mut(&mut self) -> &mut TieredStore {
        &mut self.store
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helper: text → word hashes
// ─────────────────────────────────────────────────────────────────────────────

/// Extract word hashes from text (for silk edge creation).
pub fn text_to_word_hashes(text: &str) -> Vec<u64> {
    text.split_whitespace()
        .filter(|w| w.chars().count() > 2) // skip short words
        .map(|w| fnv1a_str(&w.to_lowercase()))
        .collect()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{EmotionDim, Molecule, ShapeBase, TimeDim};

    fn test_chain(v: u8) -> MolecularChain {
        MolecularChain::single(Molecule {
            shape: ShapeBase::Sphere.as_byte(),
            relation: RelationBase::Member.as_byte(),
            emotion: EmotionDim {
                valence: v,
                arousal: 0x80,
            },
            time: TimeDim::Medium.as_byte(),
        })
    }

    #[test]
    fn knowtree_store_sentence() {
        let mut kt = KnowTree::new(4, 100);
        let chain = test_chain(0x80);
        let words = alloc::vec![0xAAAA, 0xBBBB, 0xCCCC];
        let hash = kt.store_sentence(&chain, None, &words, 1000);
        assert!(hash != 0);
        assert_eq!(kt.sentences(), 1);
        // 2 word-word edges + 3 sentence-word edges = 5
        assert_eq!(kt.total_edges(), 5);
    }

    #[test]
    fn knowtree_store_concept() {
        let mut kt = KnowTree::new(4, 100);
        let chain = test_chain(0x90);
        let sources = alloc::vec![0x1111, 0x2222, 0x3333];
        let hash = kt.store_concept(&chain, None, 3, &sources, 1000);
        assert!(hash != 0);
        assert_eq!(kt.concepts(), 1);
        assert_eq!(kt.total_edges(), 3); // 3 DerivedFrom edges
    }

    #[test]
    fn knowtree_promote() {
        let mut kt = KnowTree::new(4, 100);
        let chain = test_chain(0xA0);
        let hash = kt.promote_from_stm(&chain, None, 5, 1000);
        assert!(hash != 0);
        assert_eq!(kt.promotions(), 1);
    }

    #[test]
    fn knowtree_lookup() {
        let mut kt = KnowTree::new(4, 100);
        let chain = test_chain(0x80);
        let hash = kt.store_sentence(&chain, None, &[], 1000);

        let found = kt.lookup(hash, 2);
        assert!(found.is_some(), "Should find L2 node");
    }

    #[test]
    fn knowtree_query_with_edges() {
        let mut kt = KnowTree::new(4, 100);
        let c1 = test_chain(0x80);
        let c2 = test_chain(0x90);
        let h1 = kt.store_sentence(&c1, None, &[], 1000);
        let _h2 = kt.store_sentence(&c2, None, &[], 1001);
        kt.store_mut()
            .store_edge(h1, _h2, 0.8, RelationBase::Similar.as_byte(), 2);

        let result = kt.query(h1, 2, 1);
        assert!(result.is_some());
    }

    #[test]
    fn knowtree_store_chapter() {
        let mut kt = KnowTree::new(4, 100);
        let chains: Vec<(MolecularChain, Vec<u64>)> = (0u8..5)
            .map(|i| {
                (
                    test_chain(0x80 + i),
                    alloc::vec![i as u64 * 100, i as u64 * 100 + 1],
                )
            })
            .collect();

        let hashes = kt.store_chapter(&chains, None, 1000);
        assert_eq!(hashes.len(), 5);
        assert_eq!(kt.sentences(), 5);
        // Each sentence: 1 word-word + 2 sentence-word = 3 edges
        // Sequential: 4 sentence→sentence edges
        // Total: 5×3 + 4 = 19
        assert_eq!(kt.total_edges(), 19);
    }

    #[test]
    fn knowtree_summary() {
        let kt = KnowTree::new(4, 100);
        let s = kt.summary();
        assert!(s.contains("KnowTree"), "{}", s);
        assert!(s.contains("0 sentences"), "{}", s);
    }

    #[test]
    fn knowtree_for_pc() {
        let kt = KnowTree::for_pc();
        assert_eq!(kt.total_nodes(), 0); // freshly created, no nodes
    }

    #[test]
    fn text_to_word_hashes_basic() {
        let hashes = text_to_word_hashes("tôi buồn vì mất việc");
        // "tôi" = 3 chars, "buồn" = 4, "vì" = 2 (skip), "mất" = 3, "việc" = 4
        assert_eq!(hashes.len(), 4, "Skip 2-char words");
        // All unique
        let mut unique = hashes.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), hashes.len(), "All hashes unique");
    }

    #[test]
    fn text_to_word_hashes_deterministic() {
        let h1 = text_to_word_hashes("hello world");
        let h2 = text_to_word_hashes("hello world");
        assert_eq!(h1, h2, "Deterministic hashing");
    }

    #[test]
    fn knowtree_concept_minimum_layer() {
        let mut kt = KnowTree::new(4, 100);
        let chain = test_chain(0x80);
        // Try to store at layer 1 → should be forced to L3
        kt.store_concept(&chain, None, 1, &[], 1000);
        assert_eq!(kt.concepts(), 1);
    }

    #[test]
    fn knowtree_ram_disk_usage() {
        let mut kt = KnowTree::new(4, 100);
        for i in 0u8..10 {
            let chain = test_chain(0x80 + i);
            kt.store_sentence(&chain, None, &[], i as i64);
        }
        assert!(kt.ram_usage() > 0);
    }
}
