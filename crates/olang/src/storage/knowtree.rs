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

use crate::compact::{CompactEdge, CompactNode, NodeWithEdges, SlimNode, SlimPage, TieredStore};
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

    // ── Restore from origin.olang ──────────────────────────────────────────

    /// Restore a compact node from origin.olang bytes — boot path.
    ///
    /// QT8: origin.olang = bộ nhớ duy nhất, RAM = cache.
    pub fn restore_compact_node(&mut self, data: &[u8]) {
        if let Some(node) = CompactNode::from_bytes(data) {
            self.store.restore_node(node);
        }
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
// SlimKnowTree — spec-compliant ~10 bytes per node
// ─────────────────────────────────────────────────────────────────────────────

/// SlimKnowTree — knowledge tree dùng SlimNode/SlimPage.
///
/// Pipeline đúng spec:
/// ```text
/// UCD entry = 5 công thức (L0, bất biến)
///      │
///      ↓  input gọi công thức → tính ra giá trị
///      │
/// Molecule [S][R][V][A][T] = 5 bytes (giá trị tĩnh)
///      │
///      ↓  tagged sparse encoding
///      │
/// [mask:1B][non-defaults:0-5B] = 1-6 bytes
///      │
///      ↓  ghi vào SlimKnowTree
///      │
/// SlimNode [hash:8][tagged_len:1][tagged:1-6] = 10-15 bytes per node
/// SlimPage [header:19][nodes...][edges...][checksum:8]
/// ```
///
/// 500M nodes × 11B avg = 5.5GB → VỪA ĐIỆN THOẠI
pub struct SlimKnowTree {
    /// Pages per layer (layer → list of pages)
    pages: Vec<(u8, SlimPage)>,
    /// Current page per layer
    current_page_id: u32,
    /// Stats
    total_nodes: u64,
    total_edges: u64,
    sentences_stored: u64,
    concepts_stored: u64,
    promotions: u64,
}

impl SlimKnowTree {
    /// Create new SlimKnowTree.
    pub fn new() -> Self {
        Self {
            pages: Vec::new(),
            current_page_id: 0,
            total_nodes: 0,
            total_edges: 0,
            sentences_stored: 0,
            concepts_stored: 0,
            promotions: 0,
        }
    }

    // ── Store operations ────────────────────────────────────────────────────

    /// Store chain as SlimNode in the specified layer.
    ///
    /// Returns chain_hash.
    fn store_slim(&mut self, chain: &MolecularChain, layer: u8, ts: i64) -> u64 {
        let slim = SlimNode::from_chain(chain);
        let hash = slim.hash;

        // Find or create current page for this layer
        let page = self.current_page_mut(layer, ts);
        if !page.push_node(slim) {
            // Page full → create new page
            self.current_page_id += 1;
            let mut new_page = SlimPage::new(self.current_page_id, layer, ts);
            let slim2 = SlimNode::from_chain(chain);
            new_page.push_node(slim2);
            self.pages.push((layer, new_page));
        }

        self.total_nodes += 1;
        hash
    }

    /// Store edge in the current page for the given layer.
    fn store_edge_slim(&mut self, from: u64, to: u64, weight: f32, relation: u8, layer: u8, ts: i64) {
        let edge = CompactEdge::encode(from, to, weight, relation);
        let page = self.current_page_mut(layer, ts);
        page.push_edge(edge);
        self.total_edges += 1;
    }

    /// Store a sentence as L2 SlimNode.
    pub fn store_sentence(
        &mut self,
        chain: &MolecularChain,
        word_hashes: &[u64],
        ts: i64,
    ) -> u64 {
        let hash = self.store_slim(chain, 2, ts);
        self.sentences_stored += 1;

        // Silk edges between consecutive words
        if word_hashes.len() >= 2 {
            for w in word_hashes.windows(2) {
                self.store_edge_slim(w[0], w[1], 0.6, RelationBase::Compose.as_byte(), 2, ts);
            }
        }
        // Edge from sentence to each word
        for &wh in word_hashes {
            self.store_edge_slim(hash, wh, 0.5, RelationBase::Member.as_byte(), 2, ts);
        }

        hash
    }

    /// Store a concept (LCA result) as L3+ SlimNode.
    pub fn store_concept(
        &mut self,
        chain: &MolecularChain,
        layer: u8,
        sources: &[u64],
        ts: i64,
    ) -> u64 {
        let layer = layer.max(3);
        let hash = self.store_slim(chain, layer, ts);
        self.concepts_stored += 1;

        for &src in sources {
            self.store_edge_slim(hash, src, 0.7, RelationBase::DerivedFrom.as_byte(), layer, ts);
        }

        hash
    }

    /// Promote STM observation to L2 SlimNode.
    pub fn promote_from_stm(
        &mut self,
        chain: &MolecularChain,
        ts: i64,
    ) -> u64 {
        let hash = self.store_slim(chain, 2, ts);
        self.promotions += 1;
        hash
    }

    // ── Query operations ────────────────────────────────────────────────────

    /// Lookup SlimNode by hash in a specific layer.
    pub fn lookup(&self, hash: u64, layer: u8) -> Option<&SlimNode> {
        for (l, page) in &self.pages {
            if *l == layer {
                if let Some(node) = page.find_node(hash) {
                    return Some(node);
                }
            }
        }
        None
    }

    /// Lookup and decode to MolecularChain.
    pub fn lookup_chain(&self, hash: u64, layer: u8) -> Option<MolecularChain> {
        self.lookup(hash, layer)?.to_chain()
    }

    /// Search by text hash.
    pub fn search_text(&self, text: &str, layer: u8) -> Option<&SlimNode> {
        let hash = fnv1a_str(text);
        self.lookup(hash, layer)
    }

    // ── Batch operations ────────────────────────────────────────────────────

    /// Store chapter (batch of sentences).
    pub fn store_chapter(
        &mut self,
        chains: &[(MolecularChain, Vec<u64>)],
        ts: i64,
    ) -> Vec<u64> {
        let mut hashes = Vec::with_capacity(chains.len());
        let mut prev_hash = 0u64;

        for (chain, word_hashes) in chains {
            let hash = self.store_sentence(chain, word_hashes, ts);
            hashes.push(hash);

            if prev_hash != 0 {
                self.store_edge_slim(prev_hash, hash, 0.4, RelationBase::Causes.as_byte(), 2, ts);
            }
            prev_hash = hash;
        }

        hashes
    }

    // ── Serialization ───────────────────────────────────────────────────────

    /// Serialize all pages → bytes (for origin.olang record 0x08).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        let page_count = self.pages.len() as u32;
        buf.extend_from_slice(&page_count.to_be_bytes());
        for (_, page) in &self.pages {
            let page_bytes = page.to_bytes();
            buf.extend_from_slice(&(page_bytes.len() as u32).to_be_bytes());
            buf.extend_from_slice(&page_bytes);
        }
        buf
    }

    /// Restore from bytes.
    pub fn restore_from_bytes(&mut self, b: &[u8]) -> bool {
        if b.len() < 4 {
            return false;
        }
        let page_count = u32::from_be_bytes([b[0], b[1], b[2], b[3]]) as usize;
        let mut offset = 4;

        for _ in 0..page_count {
            if offset + 4 > b.len() {
                return false;
            }
            let page_len = u32::from_be_bytes(
                b[offset..offset + 4].try_into().unwrap_or([0; 4]),
            ) as usize;
            offset += 4;

            if offset + page_len > b.len() {
                return false;
            }
            if let Some(page) = SlimPage::from_bytes(&b[offset..offset + page_len]) {
                let layer = page.layer;
                self.total_nodes += page.nodes.len() as u64;
                self.total_edges += page.edges.len() as u64;
                self.pages.push((layer, page));
            }
            offset += page_len;
        }
        true
    }

    // ── Stats ───────────────────────────────────────────────────────────────

    /// Total nodes.
    pub fn total_nodes(&self) -> u64 {
        self.total_nodes
    }

    /// Total edges.
    pub fn total_edges(&self) -> u64 {
        self.total_edges
    }

    /// Sentences stored.
    pub fn sentences(&self) -> u64 {
        self.sentences_stored
    }

    /// Concepts stored.
    pub fn concepts(&self) -> u64 {
        self.concepts_stored
    }

    /// Promotions from STM.
    pub fn promotions(&self) -> u64 {
        self.promotions
    }

    /// Average bytes per node across all pages.
    pub fn avg_bytes_per_node(&self) -> f32 {
        if self.total_nodes == 0 {
            return 0.0;
        }
        let total_bytes: usize = self.pages.iter().map(|(_, p)| {
            p.nodes.iter().map(|n| n.total_size()).sum::<usize>()
        }).sum();
        total_bytes as f32 / self.total_nodes as f32
    }

    /// Summary.
    pub fn summary(&self) -> String {
        format!(
            "SlimKnowTree: {} sentences + {} concepts + {} promotions\n\
             Nodes: {} | Edges: {} | Pages: {} | Avg: {:.1}B/node",
            self.sentences_stored,
            self.concepts_stored,
            self.promotions,
            self.total_nodes,
            self.total_edges,
            self.pages.len(),
            self.avg_bytes_per_node(),
        )
    }

    // ── Internal ────────────────────────────────────────────────────────────

    fn current_page_mut(&mut self, layer: u8, ts: i64) -> &mut SlimPage {
        // Find existing non-full page for this layer
        let has_page = self.pages.iter().any(|(l, p)| *l == layer && !p.is_full());

        if !has_page {
            self.current_page_id += 1;
            let page = SlimPage::new(self.current_page_id, layer, ts);
            self.pages.push((layer, page));
        }

        // Return mutable reference to the last non-full page for this layer
        self.pages
            .iter_mut()
            .rev()
            .find(|(l, p)| *l == layer && !p.is_full())
            .map(|(_, p)| p)
            .unwrap()
    }
}

impl Default for SlimKnowTree {
    fn default() -> Self {
        Self::new()
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

    // ── SlimKnowTree ────────────────────────────────────────────────────────

    use super::SlimKnowTree;

    #[test]
    fn slim_knowtree_store_sentence() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0x80);
        let words = alloc::vec![0xAAAAu64, 0xBBBB, 0xCCCC];
        let hash = skt.store_sentence(&chain, &words, 1000);
        assert!(hash != 0);
        assert_eq!(skt.sentences(), 1);
        assert_eq!(skt.total_nodes(), 1);
        // 2 word-word + 3 sentence-word = 5 edges
        assert_eq!(skt.total_edges(), 5);
    }

    #[test]
    fn slim_knowtree_store_concept() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0x90);
        let sources = alloc::vec![0x1111u64, 0x2222, 0x3333];
        let hash = skt.store_concept(&chain, 3, &sources, 1000);
        assert!(hash != 0);
        assert_eq!(skt.concepts(), 1);
        assert_eq!(skt.total_edges(), 3);
    }

    #[test]
    fn slim_knowtree_promote() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0xA0);
        let hash = skt.promote_from_stm(&chain, 1000);
        assert!(hash != 0);
        assert_eq!(skt.promotions(), 1);
    }

    #[test]
    fn slim_knowtree_lookup() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0x80);
        let hash = skt.store_sentence(&chain, &[], 1000);

        let found = skt.lookup(hash, 2);
        assert!(found.is_some(), "Should find L2 slim node");
        assert_eq!(found.unwrap().hash, hash);
    }

    #[test]
    fn slim_knowtree_lookup_chain() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0xB0);
        let hash = skt.store_sentence(&chain, &[], 1000);

        let decoded = skt.lookup_chain(hash, 2);
        assert!(decoded.is_some());
        assert_eq!(decoded.unwrap(), chain);
    }

    #[test]
    fn slim_knowtree_store_chapter() {
        let mut skt = SlimKnowTree::new();
        let chains: Vec<(MolecularChain, Vec<u64>)> = (0u8..5)
            .map(|i| {
                (
                    test_chain(0x80 + i),
                    alloc::vec![i as u64 * 100, i as u64 * 100 + 1],
                )
            })
            .collect();

        let hashes = skt.store_chapter(&chains, 1000);
        assert_eq!(hashes.len(), 5);
        assert_eq!(skt.sentences(), 5);
        // Each: 1 word-word + 2 sentence-word = 3. Sequential: 4.
        // Total: 5×3 + 4 = 19
        assert_eq!(skt.total_edges(), 19);
    }

    #[test]
    fn slim_knowtree_serialization_roundtrip() {
        let mut skt = SlimKnowTree::new();
        for i in 0u8..10 {
            let chain = test_chain(0x80 + i);
            skt.store_sentence(&chain, &[], i as i64);
        }

        let bytes = skt.to_bytes();
        let mut restored = SlimKnowTree::new();
        assert!(restored.restore_from_bytes(&bytes));
        assert_eq!(restored.total_nodes(), 10);
    }

    #[test]
    fn slim_knowtree_avg_bytes_per_node() {
        let mut skt = SlimKnowTree::new();
        for i in 0u8..50 {
            let chain = test_chain(0x80 + (i % 20));
            skt.store_sentence(&chain, &[], i as i64);
        }
        let avg = skt.avg_bytes_per_node();
        // SlimNode: hash:8 + len:1 + tagged:2-3 = ~11-12 bytes
        assert!(
            avg <= 15.0,
            "Average {:.1}B/node should be ≤ 15B",
            avg
        );
    }

    #[test]
    fn slim_knowtree_summary() {
        let skt = SlimKnowTree::new();
        let s = skt.summary();
        assert!(s.contains("SlimKnowTree"), "{}", s);
    }

    #[test]
    fn slim_knowtree_concept_minimum_layer() {
        let mut skt = SlimKnowTree::new();
        let chain = test_chain(0x80);
        // Try layer 1 → forced to L3
        skt.store_concept(&chain, 1, &[], 1000);
        assert_eq!(skt.concepts(), 1);
        // Should be findable at layer 3
        let hash = chain.chain_hash();
        assert!(skt.lookup(hash, 3).is_some());
    }
}
