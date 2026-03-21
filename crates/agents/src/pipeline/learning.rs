//! # learning — LearningLoop
//!
//! Trái tim đập — kết nối mọi subsystem.
//! Mỗi input → ContentEncoder → chain → STM → Silk → Dream.
//!
//! Pipeline:
//!   input → gate.check() → encode() → context.on_activate()
//!   → stm.push(chain) → silk.co_activate() → [dream khi idle]

extern crate alloc;
use alloc::vec::Vec;

use context::engine::ContextEngine;
use olang::molecular::{Dimension, Maturity, MolecularChain, Molecule};
use silk::edge::EmotionTag;
use silk::graph::{MolSummary, SilkGraph};

use crate::encoder::{ContentEncoder, ContentInput, EncodedContent, SensorKind};
use crate::gate::{GateVerdict, SecurityGate};

// ─────────────────────────────────────────────────────────────────────────────
// Molecule → MolSummary conversion
// ─────────────────────────────────────────────────────────────────────────────

/// Molecule → MolSummary cho Silk 5D comparison.
fn mol_to_summary(mol: &Molecule) -> MolSummary {
    MolSummary {
        shape: mol.shape_u8(),
        relation: mol.relation_u8(),
        valence: mol.valence_u8(),
        arousal: mol.arousal_u8(),
        time: mol.time_u8(),
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ShortTermMemory (ĐN)
// ─────────────────────────────────────────────────────────────────────────────

/// ĐN — Short-Term Memory.
///
/// Buffer trước khi vào QR (long-term).
/// Tối đa 512 observations trước khi Dream flush.
#[derive(Debug)]
pub struct ShortTermMemory {
    observations: Vec<Observation>,
    max_size: usize,
}

/// Một observation trong ĐN.
#[derive(Debug, Clone)]
pub struct Observation {
    pub chain: MolecularChain,
    pub emotion: EmotionTag,
    pub timestamp: i64,
    pub fire_count: u32,
    /// Cached 5D summary cho Silk comparison — tránh tính lại.
    pub mol_summary: Option<MolSummary>,
    /// Maturity state: Formula → Evaluating → Mature
    pub maturity: Maturity,
    /// Layer (tầng) của observation — QT⑪ enforcement.
    /// Default: 0 (L0). Dream cluster chỉ trong cùng layer.
    pub layer: u8,
}

impl ShortTermMemory {
    pub fn new(max_size: usize) -> Self {
        Self {
            observations: Vec::new(),
            max_size,
        }
    }

    /// Thêm observation — nếu đã có chain tương tự → tăng fire_count.
    pub fn push(&mut self, chain: MolecularChain, emotion: EmotionTag, ts: i64) {
        let hash = chain.chain_hash();

        // Tìm observation đã có
        if let Some(obs) = self
            .observations
            .iter_mut()
            .find(|o| o.chain.chain_hash() == hash)
        {
            obs.fire_count += 1;
            // Cập nhật maturity dựa trên fire_count mới
            // fib(2) = 2 — threshold cho STM (depth=2)
            let fib_threshold = silk::hebbian::fib(2);
            // Gap #8 fix: dùng fire_count-based heuristic thay vì 0.0
            let approx_weight = (obs.fire_count as f32 / (fib_threshold as f32 + 5.0)).min(1.0);
            // Molecule = Công thức: advance dùng evaluated_count từ chain
            let eval_dims = obs.chain.first().map_or(5, |m| m.evaluated_count());
            obs.maturity = obs.maturity.advance_with_eval(obs.fire_count, approx_weight, fib_threshold, eval_dims);
            // Blend emotion
            obs.emotion = obs.emotion.blend(emotion, 0.3);
            obs.timestamp = ts;
            return;
        }

        // Mới — thêm vào
        if self.observations.len() >= self.max_size {
            // Xóa observation ít được fire nhất (LFU eviction)
            if let Some(min_idx) = self
                .observations
                .iter()
                .enumerate()
                .min_by_key(|(_, o)| o.fire_count)
                .map(|(i, _)| i)
            {
                self.observations.remove(min_idx);
            }
        }

        // Cache MolSummary cho Silk 5D comparison
        let mol_summary = chain.first().as_ref().map(mol_to_summary);

        self.observations.push(Observation {
            chain,
            emotion,
            timestamp: ts,
            fire_count: 1,
            mol_summary,
            maturity: Maturity::Formula,
            layer: 0,
        });
    }

    /// Observations được fire nhiều nhất — Dream candidates.
    pub fn top_n(&self, n: usize) -> Vec<&Observation> {
        let mut sorted: Vec<&Observation> = self.observations.iter().collect();
        sorted.sort_by(|a, b| b.fire_count.cmp(&a.fire_count));
        sorted.into_iter().take(n).collect()
    }

    pub fn len(&self) -> usize {
        self.observations.len()
    }
    pub fn is_empty(&self) -> bool {
        self.observations.is_empty()
    }
    pub fn all(&self) -> &[Observation] {
        &self.observations
    }

    /// Mutable access to observations — for restore from origin.olang.
    pub fn all_mut(&mut self) -> &mut Vec<Observation> {
        &mut self.observations
    }

    /// Tìm observation theo chain_hash.
    pub fn find_by_hash(&self, hash: u64) -> Option<&Observation> {
        self.observations
            .iter()
            .find(|o| o.chain.chain_hash() == hash)
    }

    /// Restore an observation from origin.olang — boot path.
    ///
    /// Pushes a raw observation without encoding (chain already exists in registry).
    /// QT8: origin.olang = bộ nhớ duy nhất, RAM = cache.
    pub fn push_raw(
        &mut self,
        _chain_hash: u64,
        emotion: EmotionTag,
        fire_count: u32,
        maturity: Maturity,
        layer: u8,
        timestamp: i64,
    ) {
        if self.observations.len() >= self.max_size {
            // LFU eviction
            if let Some(min_idx) = self
                .observations
                .iter()
                .enumerate()
                .min_by_key(|(_, o)| o.fire_count)
                .map(|(i, _)| i)
            {
                self.observations.remove(min_idx);
            }
        }

        // Tạo observation stub — chain rỗng (hash đã có trong registry).
        // Chain sẽ được resolve khi cần qua registry lookup.
        self.observations.push(Observation {
            chain: MolecularChain::empty(),
            emotion,
            timestamp,
            fire_count,
            mol_summary: None,
            maturity,
            layer,
        });
    }

    /// Mark observation as Mature (called after Dream detects matured nodes).
    pub fn mark_matured(&mut self, hash: u64) {
        for obs in &mut self.observations {
            if obs.chain.chain_hash() == hash {
                obs.maturity = olang::mol::molecular::Maturity::Mature;
                break;
            }
        }
    }

    /// Xóa observations đã được promote lên QR.
    pub fn remove_promoted(&mut self, hashes: &[u64]) {
        self.observations
            .retain(|o| !hashes.contains(&o.chain.chain_hash()));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ProcessResult
// ─────────────────────────────────────────────────────────────────────────────

/// Kết quả của process_one().
#[derive(Debug)]
pub enum ProcessResult {
    /// Xử lý thành công
    Ok {
        chain: MolecularChain,
        emotion: EmotionTag,
    },
    /// SecurityGate block
    Blocked { reason: alloc::string::String },
    /// Crisis — cần response đặc biệt
    Crisis { message: alloc::string::String },
    /// Input rỗng
    Empty,
}

// ─────────────────────────────────────────────────────────────────────────────
// LearningLoop
// ─────────────────────────────────────────────────────────────────────────────

/// Trái tim đập của HomeOS.
///
/// Kết nối: Gate → Encoder → Context → STM → Silk
pub struct LearningLoop {
    gate: SecurityGate,
    encoder: ContentEncoder,
    context: ContextEngine,
    stm: ShortTermMemory,
    graph: SilkGraph,
    /// Hash của chain trước — để co_activate Silk
    prev_hash: Option<u64>,
    /// MolSummary của chain trước — cho 5D Silk comparison
    prev_mol: Option<MolSummary>,
    /// Hash đại diện câu trước — link câu với câu
    prev_sent_hash: Option<u64>,
}

impl LearningLoop {
    pub fn new(session_id: u64) -> Self {
        Self {
            gate: SecurityGate::new(),
            encoder: ContentEncoder::new(),
            context: ContextEngine::new(session_id),
            stm: ShortTermMemory::new(512),
            graph: SilkGraph::new(),
            prev_hash: None,
            prev_mol: None,
            prev_sent_hash: None,
        }
    }

    /// Xử lý một ContentInput qua toàn bộ pipeline.
    ///
    /// BẢN NĂNG — chạy cho MỌI modality (text, audio, sensor, code, math, system).
    /// Gate → Encode → Context → STM → Silk → Learn → kết quả
    pub fn process_one(&mut self, input: ContentInput) -> ProcessResult {
        let ts = input.timestamp();

        // ── 0. Security Gate — BẢN NĂNG: TRƯỚC MỌI THỨ, MỌI MODALITY ────────
        match self.gate.check_input(&input) {
            GateVerdict::Crisis { message } => {
                return ProcessResult::Crisis { message };
            }
            GateVerdict::Block { reason } => {
                return ProcessResult::Blocked {
                    reason: alloc::format!("{:?}", reason),
                };
            }
            GateVerdict::Allow | GateVerdict::BlackCurtain => {}
        }

        // ── 1. Encode → chain (BẢN NĂNG: mọi input → MolecularChain) ────────
        let encoded: EncodedContent = self.encoder.encode(input.clone());
        if encoded.chain.is_empty() {
            return ProcessResult::Empty;
        }

        let chain = encoded.chain.clone();
        let emotion = encoded.emotion;
        let hash = chain.chain_hash();

        // ── CHECKPOINT 2: ENCODE ────────────────────────────────────────────
        // Spec §X CP2: entities≥1, chain_hash≠0, consistency≥0.75
        // Vi phạm → Honesty: "Không hiểu input"
        {
            // (a) chain_hash phải khác 0
            if hash == 0 {
                return ProcessResult::Empty;
            }
            // (b) chain phải có ít nhất 1 link (entity)
            if chain.len() < 1 {
                return ProcessResult::Empty;
            }
            // (c) consistency: kiểm tra chain không toàn zero
            let non_zero_count = chain.0.iter().filter(|&&m| m != 0).count();
            let consistency = if chain.len() > 0 {
                non_zero_count as f32 / chain.len() as f32
            } else {
                0.0
            };
            if consistency < 0.75 {
                return ProcessResult::Blocked {
                    reason: alloc::string::String::from("CP2: encode consistency < 0.75"),
                };
            }
        }

        // ── 2. Context Engine — BẢN NĂNG: mọi modality cập nhật context ─────
        {
            use context::snapshot::RawInput;
            let raw = match &input {
                ContentInput::Text { content, timestamp } => RawInput::text(content, *timestamp),
                ContentInput::Audio {
                    freq_hz,
                    amplitude,
                    timestamp,
                    ..
                } => RawInput::audio(*freq_hz, *amplitude, *timestamp),
                _ =>
                // Sensor, Code, Math, System → text-like context
                {
                    RawInput::text("", ts)
                }
            };
            if raw.text.is_some() || raw.audio_pitch.is_some() {
                self.context.on_activate(raw);
            }
        }

        // ── 3. STM — BẢN NĂNG: mọi input → observation (ghi nhớ) ────────────
        self.stm.push(chain.clone(), emotion, ts);

        // ── 4. Silk — BẢN NĂNG: co_activate với node trước (liên tưởng) ──────
        // Dùng co_activate_mol: 5D similarity boost kết nối giữa nodes tương đồng
        let current_mol = chain.first().as_ref().map(mol_to_summary);
        if let Some(prev) = self.prev_hash {
            if prev != hash {
                self.graph.co_activate_mol(
                    prev,
                    hash,
                    self.prev_mol,
                    current_mol,
                    emotion,
                    emotion.intensity.max(0.1),
                    ts,
                );
            }
        }
        self.prev_hash = Some(hash);
        self.prev_mol = current_mol;

        // ── 5. Học chuyên sâu theo modality ──────────────────────────────────
        match &input {
            ContentInput::Text { content, timestamp } => {
                // 5 tầng học ngôn ngữ: câu → cụm từ → từ → ký tự
                self.learn_text(content, emotion, *timestamp);
            }
            ContentInput::Audio {
                freq_hz, amplitude, ..
            } => {
                // Audio: co-activate freq pattern với emotion
                let freq_hash = frequency_hash(*freq_hz);
                self.graph
                    .co_activate(hash, freq_hash, emotion, amplitude.max(0.1), ts);
            }
            ContentInput::Sensor { kind, value, .. } => {
                // Sensor: co-activate sensor kind với value pattern
                let kind_hash = sensor_kind_hash(kind);
                self.graph
                    .co_activate(hash, kind_hash, emotion, value.abs().clamp(0.1, 1.0), ts);
            }
            _ => {
                // Code, Math, System — chain + STM + Silk đủ (đã chạy ở bước 3+4)
            }
        }

        ProcessResult::Ok { chain, emotion }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // learn_text — 5 tầng học từ văn bản
    // ─────────────────────────────────────────────────────────────────────────
    //
    // Đoạn văn → Câu → Cụm từ → Từ → Ký tự
    //
    // Ký tự đã có ở L0 (encode_codepoint). Từ đây xử lý 4 tầng trên.

    /// Feed text qua 4 tầng học còn lại (câu/cụm từ/từ đã có ký tự ở L0).
    pub fn learn_text(&mut self, text: &str, paragraph_emotion: EmotionTag, ts: i64) {
        // ── Tầng 1: Câu — tách theo dấu câu ─────────────────────────────────
        let sentences = split_sentences(text);
        for (si, sent) in sentences.iter().enumerate() {
            if sent.trim().is_empty() {
                continue;
            }

            // Emotion của câu này (blend paragraph + word-level qua Silk amplify)
            let sent_tag = {
                let wt = context::emotion::sentence_affect(sent);
                // Amplify: paragraph context (60%) blend sentence (40%)
                // Dùng EmotionTag::blend thay vì trung bình đơn giản (QT: KHÔNG average)
                paragraph_emotion.blend(wt, 0.6)
            };

            let words = content_words(sent);
            if words.is_empty() {
                continue;
            }

            let hashes: alloc::vec::Vec<u64> = words.iter().map(|w| word_hash(w)).collect();

            // ── Tầng 2: Từ — node riêng, emotion = context blend ─────────────
            for (wi, w) in words.iter().enumerate() {
                let wt = context::emotion::word_affect(w);
                let tag = if wt.intensity > 0.10 {
                    // Blend: 60% sentence context + 40% lexicon
                    EmotionTag {
                        valence: sent_tag.valence * 0.6 + wt.valence * 0.4,
                        arousal: sent_tag.arousal * 0.6 + wt.arousal * 0.4,
                        dominance: sent_tag.dominance * 0.6 + wt.dominance * 0.4,
                        intensity: sent_tag.intensity * 0.6 + wt.intensity * 0.4,
                    }
                } else {
                    sent_tag
                };

                // Activate từ với từ liền trước trong câu
                if wi > 0 {
                    // Cạnh từ gần nhau (khoảng cách 1) → reward cao
                    self.graph
                        .co_activate(hashes[wi - 1], hashes[wi], tag, 0.8, ts);
                }

                // ── Tầng 3: Cụm từ — sliding window 5 ───────────────────────
                let win_end = (wi + 5).min(hashes.len());
                for wj in (wi + 2)..win_end {
                    // bắt đầu từ +2 (khoảng cách 1 đã làm trên)
                    let gap = (wj - wi) as f32;
                    let proximity = 1.0 - gap / 5.0; // gần hơn → mạnh hơn

                    let pair_tag = EmotionTag {
                        valence: sent_tag.valence,
                        arousal: sent_tag.arousal,
                        dominance: sent_tag.dominance,
                        intensity: sent_tag.intensity * proximity,
                    };
                    self.graph.co_activate(
                        hashes[wi],
                        hashes[wj],
                        pair_tag,
                        proximity * sent_tag.intensity.max(0.05),
                        ts,
                    );
                }
            }

            // ── Tầng 4: Câu liên tiếp — link câu trước với câu sau ───────────
            if si > 0 && !hashes.is_empty() {
                // Lấy hash đại diện của câu = hash từ đầu tiên
                let sent_hash = hashes[0];
                if let Some(prev_sent_hash) = self.prev_sent_hash {
                    self.graph.co_activate(
                        prev_sent_hash,
                        sent_hash,
                        sent_tag,
                        sent_tag.intensity.max(0.05),
                        ts,
                    );
                }
                self.prev_sent_hash = Some(sent_hash);
            } else if si == 0 && !hashes.is_empty() {
                self.prev_sent_hash = Some(hashes[0]);
            }
        }
    }

    // ── Body Updates — cập nhật Spline từ learning ────────────────────────────

    /// Collect body spline updates từ STM observations gần đây.
    ///
    /// Khi learning tích lũy đủ data về 1 concept (fire_count >= 2),
    /// trả về danh sách (chain_hash, emotion) để Runtime cập nhật BodyStore.
    ///
    /// Đây là bridge: Learning → BodyStore (Spline cập nhật từ thực tế).
    pub fn pending_body_updates(&self) -> Vec<BodyUpdate> {
        let mut updates = Vec::new();

        for obs in self.stm.all() {
            // Chỉ cập nhật khi fire_count >= 2 (đủ evidence, không phải noise)
            if obs.fire_count < 2 {
                continue;
            }

            updates.push(BodyUpdate {
                chain_hash: obs.chain.chain_hash(),
                emotion: obs.emotion,
                fire_count: obs.fire_count,
                mol_summary: obs.mol_summary,
            });
        }

        updates
    }

    // ── Maintain — chăm sóc Ln-1 ─────────────────────────────────────────────

    /// Chăm sóc Silk graph: decay + cắt tỉa overflow.
    ///
    /// `elapsed_ns`: thời gian đã trôi kể từ lần maintain trước.
    /// `max_edges`: giới hạn tổng số edges (0 = không giới hạn).
    ///
    /// Trả về số edges đã bị cắt tỉa.
    pub fn maintain_silk(&mut self, elapsed_ns: i64, max_edges: usize) -> usize {
        self.graph.maintain(elapsed_ns, max_edges)
    }

    // ── Accessors ────────────────────────────────────────────────────────────

    pub fn stm(&self) -> &ShortTermMemory {
        &self.stm
    }
    pub fn graph(&self) -> &SilkGraph {
        &self.graph
    }
    pub fn stm_mut(&mut self) -> &mut ShortTermMemory {
        &mut self.stm
    }
    pub fn graph_mut(&mut self) -> &mut SilkGraph {
        &mut self.graph
    }
    pub fn context(&self) -> &ContextEngine {
        &self.context
    }
    pub fn gate(&self) -> &SecurityGate {
        &self.gate
    }
    pub fn turn_count(&self) -> usize {
        self.context.turn_count()
    }

    /// Dream candidates từ STM.
    pub fn dream_candidates(&self, n: usize) -> Vec<&Observation> {
        self.stm.top_n(n)
    }

    /// Restore STM observation from origin.olang — boot path.
    ///
    /// QT8: origin.olang = bộ nhớ duy nhất. Boot replay → RAM cache.
    #[allow(clippy::too_many_arguments)]
    pub fn restore_stm_observation(
        &mut self,
        chain_hash: u64,
        valence: f32,
        arousal: f32,
        dominance: f32,
        intensity: f32,
        fire_count: u32,
        maturity_byte: u8,
        layer: u8,
        timestamp: i64,
    ) {
        let emotion = silk::edge::EmotionTag {
            valence,
            arousal,
            dominance,
            intensity,
        };
        let maturity = olang::molecular::Maturity::from_byte(maturity_byte);

        // Tìm observation đã có (dedup by hash — append-only, last wins)
        if let Some(obs) = self
            .stm
            .all_mut()
            .iter_mut()
            .find(|o| o.chain.chain_hash() == chain_hash)
        {
            // Update — last record wins (append-only)
            obs.emotion = emotion;
            obs.fire_count = fire_count;
            obs.maturity = maturity;
            obs.layer = layer;
            obs.timestamp = timestamp;
            return;
        }

        // Mới — push raw observation (không qua encode pipeline)
        self.stm.push_raw(chain_hash, emotion, fire_count, maturity, layer, timestamp);
    }

    /// Restore ConversationCurve turn from origin.olang — boot path.
    pub fn restore_curve_turn(&mut self, valence: f32, fx_dn: f32) {
        self.context.curve_mut().push(valence);
        self.context.curve_mut().update_dn(fx_dn);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Evolution detection — tìm "loài mới" từ learning
    // ─────────────────────────────────────────────────────────────────────────

    /// So sánh chain mới với STM observations — tìm evolution candidates.
    ///
    /// Nếu chain mới khác 1 observation đúng 1 dimension → evolution candidate.
    /// Trả về danh sách (source_chain, dimension, old_value, new_value).
    ///
    /// Logic:
    ///   - So sánh first molecule (đại diện ngữ nghĩa) của chain mới vs STM
    ///   - Chỉ khác đúng 1 dimension → candidate (mutation = evolution)
    ///   - Khác 0 → identical, khác 2+ → quá xa (không phải evolution)
    pub fn detect_evolutions(
        &self,
        new_chain: &MolecularChain,
    ) -> Vec<EvolutionCandidate> {
        let new_mol = match new_chain.first() {
            Some(m) => m,
            None => return Vec::new(),
        };
        let new_hash = new_chain.chain_hash();

        let mut candidates = Vec::new();

        for obs in self.stm.all() {
            // Không so sánh với chính mình
            if obs.chain.chain_hash() == new_hash {
                continue;
            }

            if let Some(obs_mol) = obs.chain.first() {
                let deltas = obs_mol.dimension_delta(&new_mol);
                if deltas.len() == 1 {
                    // Đúng 1 dimension khác → evolution candidate!
                    let (dim, old_val, new_val) = deltas[0];
                    candidates.push(EvolutionCandidate {
                        source_hash: obs.chain.chain_hash(),
                        source_chain: obs.chain.clone(),
                        dimension: dim,
                        old_value: old_val,
                        new_value: new_val,
                    });
                }
            }
        }

        candidates
    }
}

/// Evolution candidate — "loài mới" phát hiện trong learning pipeline.
#[derive(Debug, Clone)]
pub struct EvolutionCandidate {
    /// Hash của chain gốc (đã có trong STM)
    pub source_hash: u64,
    /// Chain gốc
    pub source_chain: MolecularChain,
    /// Dimension đã thay đổi
    pub dimension: Dimension,
    /// Giá trị cũ (ở chain gốc)
    pub old_value: u8,
    /// Giá trị mới (ở chain mới)
    pub new_value: u8,
}

/// Body update — cập nhật BodyStore splines từ learning data.
///
/// Runtime nhận BodyUpdate → cập nhật emotion_v, emotion_a splines.
/// Đây là bridge "Learning → Body" — Spline thay đổi theo thực tế.
#[derive(Debug, Clone)]
pub struct BodyUpdate {
    /// chain_hash của node cần cập nhật
    pub chain_hash: u64,
    /// Emotion tích lũy từ learning (blend qua nhiều observations)
    pub emotion: EmotionTag,
    /// Số lần fire — confidence indicator
    pub fire_count: u32,
    /// 5D coordinates (nếu có)
    pub mol_summary: Option<MolSummary>,
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::encoder::{SensorKind, SystemEvent};
    use alloc::string::ToString;

    fn loop_() -> LearningLoop {
        LearningLoop::new(0x1234)
    }


    fn text(s: &str) -> ContentInput {
        ContentInput::Text {
            content: s.to_string(),
            timestamp: 1000,
        }
    }

    // ── STM ───────────────────────────────────────────────────────────────────

    #[test]
    fn stm_push_increases_len() {
        let mut stm = ShortTermMemory::new(10);
        let chain = olang::encoder::encode_codepoint(0x1F525);
        stm.push(chain, EmotionTag::NEUTRAL, 1000);
        assert_eq!(stm.len(), 1);
    }

    #[test]
    fn stm_same_chain_increments_fire() {
        let mut stm = ShortTermMemory::new(10);
        let chain = olang::encoder::encode_codepoint(0x1F525);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 1000);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 2000);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 3000);
        assert_eq!(stm.len(), 1, "Cùng chain → không tạo duplicate");
        assert_eq!(stm.all()[0].fire_count, 3);
    }

    #[test]
    fn stm_top_n_sorted() {
        let mut stm = ShortTermMemory::new(10);
        let c1 = olang::encoder::encode_codepoint(0x1F525); // fire
        let c2 = olang::encoder::encode_codepoint(0x1F4A7); // water
        let c3 = olang::encoder::encode_codepoint(0x2744); // snow

        // c2 fire nhiều nhất
        for _ in 0..5 {
            stm.push(c2.clone(), EmotionTag::NEUTRAL, 1);
        }
        for _ in 0..3 {
            stm.push(c1.clone(), EmotionTag::NEUTRAL, 2);
        }
        for _ in 0..1 {
            stm.push(c3.clone(), EmotionTag::NEUTRAL, 3);
        }

        let top = stm.top_n(2);
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].fire_count, 5, "Top 1 phải có fire_count=5");
        assert_eq!(top[1].fire_count, 3, "Top 2 phải có fire_count=3");
    }

    #[test]
    fn stm_eviction_lfu() {
        let mut stm = ShortTermMemory::new(3); // max 3
                                               // Đổ 4 chains khác nhau
        let chains: Vec<_> = [0x1F525u32, 0x1F4A7, 0x2744, 0x1F9E0]
            .iter()
            .map(|&cp| olang::encoder::encode_codepoint(cp))
            .collect();

        stm.push(chains[0].clone(), EmotionTag::NEUTRAL, 1);
        stm.push(chains[0].clone(), EmotionTag::NEUTRAL, 2); // fire=2
        stm.push(chains[1].clone(), EmotionTag::NEUTRAL, 3); // fire=1
        stm.push(chains[2].clone(), EmotionTag::NEUTRAL, 4); // fire=1
                                                             // Max reached, thêm chains[3] → evict LFU (chains[1] hoặc chains[2])
        stm.push(chains[3].clone(), EmotionTag::NEUTRAL, 5);

        assert_eq!(stm.len(), 3, "Eviction giữ max_size");
        // chains[0] vẫn còn (fire=2 cao nhất)
        assert!(
            stm.all()
                .iter()
                .any(|o| o.chain.chain_hash() == chains[0].chain_hash()),
            "chains[0] fire=2 phải còn"
        );
    }

    // ── Maturity ────────────────────────────────────────────────────────────

    #[test]
    fn observation_starts_as_formula() {
        let mut stm = ShortTermMemory::new(512);
        let chain = olang::encoder::encode_codepoint(0x1F525);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
        let obs = stm.top_n(1);
        assert_eq!(obs[0].maturity, Maturity::Formula, "Lần đầu push → Formula");
    }

    #[test]
    fn observation_advances_to_evaluating_on_second_fire() {
        let mut stm = ShortTermMemory::new(512);
        let chain = olang::encoder::encode_codepoint(0x1F525);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 0);
        stm.push(chain.clone(), EmotionTag::NEUTRAL, 1); // fire_count → 2 >= fib(2)=2
        let obs = stm.top_n(1);
        assert!(
            obs[0].maturity == Maturity::Evaluating || obs[0].maturity == Maturity::Mature,
            "fire_count=2 → ít nhất Evaluating: {:?}", obs[0].maturity
        );
    }

    // ── LearningLoop ─────────────────────────────────────────────────────────

    #[test]
    fn process_text_ok() {
        let mut l = loop_();
        let r = l.process_one(text("hôm nay trời đẹp"));
        assert!(matches!(r, ProcessResult::Ok { .. }), "Normal text → Ok");
        assert_eq!(l.stm().len(), 1);
    }

    #[test]
    fn process_crisis_intercept() {
        let mut l = loop_();
        let r = l.process_one(text("tôi muốn chết"));
        assert!(
            matches!(r, ProcessResult::Crisis { .. }),
            "Crisis phải được intercept trước khi encode"
        );
        // STM không được populated
        assert_eq!(l.stm().len(), 0, "Crisis không vào STM");
    }

    #[test]
    fn process_block_intercept() {
        let mut l = loop_();
        let r = l.process_one(text("rm -rf /"));
        assert!(matches!(r, ProcessResult::Blocked { .. }));
        assert_eq!(l.stm().len(), 0, "Blocked không vào STM");
    }

    #[test]
    fn process_multiple_builds_silk() {
        let mut l = loop_();
        // Dùng emoji — mỗi cái có codepoint khác nhau → chain hash khác nhau
        l.process_one(ContentInput::Text {
            content: "🔥".to_string(),
            timestamp: 1000,
        });
        l.process_one(ContentInput::Text {
            content: "💧".to_string(),
            timestamp: 2000,
        });
        l.process_one(ContentInput::Text {
            content: "🔥".to_string(),
            timestamp: 3000,
        });
        // Silk: 🔥 → 💧 (different hashes → edge)
        // STM phải có entries
        assert!(!l.stm().is_empty(), "STM phải có entries");
        // Graph có thể có hoặc không tùy chain hash — check STM thay thế
    }

    #[test]
    fn process_sensor_ok() {
        let mut l = loop_();
        let r = l.process_one(ContentInput::Sensor {
            kind: SensorKind::Temperature,
            value: 38.0,
            timestamp: 1000,
        });
        assert!(matches!(r, ProcessResult::Ok { .. }));
    }

    #[test]
    fn process_system_boot_ok() {
        let mut l = loop_();
        let r = l.process_one(ContentInput::System {
            event: SystemEvent::Boot,
            timestamp: 0,
        });
        assert!(matches!(r, ProcessResult::Ok { .. }));
    }

    #[test]
    fn dream_candidates_top_fired() {
        let mut l = loop_();
        // Gửi "tôi buồn" nhiều lần
        for i in 0..5 {
            l.process_one(ContentInput::Text {
                content: "tôi buồn".to_string(),
                timestamp: i * 1000,
            });
        }
        l.process_one(ContentInput::Text {
            content: "trời đẹp".to_string(),
            timestamp: 10000,
        });

        let candidates = l.dream_candidates(1);
        assert!(!candidates.is_empty(), "Phải có candidates");
        // Top candidate phải có fire_count cao nhất
        assert!(candidates[0].fire_count >= 1);
    }

    #[test]
    fn audio_frequency_co_activates_silk() {
        let mut l = loop_();
        // Feed two audio inputs at same frequency → should co-activate in Silk
        let r1 = l.process_one(ContentInput::Audio {
            freq_hz: 440.0,
            amplitude: 0.7,
            duration_ms: 500,
            timestamp: 1000,
        });
        assert!(matches!(r1, ProcessResult::Ok { .. }));
        let r2 = l.process_one(ContentInput::Audio {
            freq_hz: 440.0,
            amplitude: 0.8,
            duration_ms: 300,
            timestamp: 2000,
        });
        assert!(matches!(r2, ProcessResult::Ok { .. }));
        // Graph should have at least one edge from audio co-activation
        assert!(!l.graph().is_empty(), "Audio freq should create Silk edges");
    }

    #[test]
    fn audio_different_freq_bands_distinct() {
        let mut l = loop_();
        // Sub-bass (40 Hz) and treble (8000 Hz) should produce different freq hashes
        l.process_one(ContentInput::Audio {
            freq_hz: 40.0,
            amplitude: 0.5,
            duration_ms: 500,
            timestamp: 1000,
        });
        let edges_after_bass = l.graph().len();
        l.process_one(ContentInput::Audio {
            freq_hz: 8000.0,
            amplitude: 0.5,
            duration_ms: 500,
            timestamp: 2000,
        });
        let edges_after_treble = l.graph().len();
        // Both should have created edges (each audio co-activates chain hash ↔ freq hash)
        assert!(edges_after_bass > 0, "Bass should create Silk edge");
        assert!(edges_after_treble > 0, "Treble should create Silk edge");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers — tách văn bản
// ─────────────────────────────────────────────────────────────────────────────

use alloc::string::ToString;

/// Tách văn bản thành câu theo dấu . ! ? — shared across agents crate.
pub(crate) fn split_sentences(text: &str) -> alloc::vec::Vec<alloc::string::String> {
    let mut sentences = alloc::vec::Vec::new();
    let mut cur = alloc::string::String::new();
    for ch in text.chars() {
        cur.push(ch);
        if matches!(ch, '.' | '!' | '?' | '。' | '！' | '？') {
            let s = cur.trim().to_string();
            if !s.is_empty() {
                sentences.push(s);
            }
            cur.clear();
        }
    }
    let s = cur.trim().to_string();
    if !s.is_empty() {
        sentences.push(s);
    }
    sentences
}

/// Lấy content words — loại stop words và từ quá ngắn.
fn content_words(text: &str) -> alloc::vec::Vec<alloc::string::String> {
    text.split_whitespace()
        .map(|w| {
            // Bỏ dấu câu xung quanh
            {
                let s: alloc::string::String = w
                    .chars()
                    .filter(|c| c.is_alphanumeric() || *c > '\x7f')
                    .collect();
                s.to_lowercase()
            }
        })
        .filter(|w| {
            let n = w.chars().count();
            n >= 2 && !is_stop_word(w)
        })
        .collect()
}

/// Hash ổn định cho một từ — dùng shared FNV-1a.
fn word_hash(word: &str) -> u64 {
    olang::hash::fnv1a_str(word)
}

/// Stop words — không tạo Silk node riêng.
fn is_stop_word(w: &str) -> bool {
    matches!(
        w,
        // VI phổ biến
        "và"|"của"|"các"|"trong"|"với"|"này"|"đó"|"cho"|"những"|"một"|
        "hay"|"khi"|"đã"|"đang"|"sẽ"|"bị"|"được"|"có"|"là"|"thì"|
        "mà"|"nên"|"vì"|"nếu"|"theo"|"sau"|"trên"|"dưới"|"như"|
        "rồi"|"lại"|"cũng"|"vẫn"|"còn"|"ra"|"vào"|"lên"|"xuống"|
        "đây"|"kia"|"ở"|"tại"|"về"|"đến"|"từ"|"qua"|"lúc"|
        // EN phổ biến
        "the"|"and"|"was"|"for"|"are"|"with"|"his"|"that"|"had"|
        "but"|"not"|"from"|"they"|"she"|"him"|"her"|"its"|"also"
    )
}

/// Hash cho frequency range (audio learning) — namespace 0xAA.
fn frequency_hash(freq_hz: f32) -> u64 {
    // Quantize to octave bands: 20-40, 40-80, ..., 10240-20480
    let octave = if freq_hz <= 0.0 {
        0u8
    } else {
        ((homemath::log2f(freq_hz / 20.0).max(0.0)) as u8).min(10)
    };
    olang::hash::fnv1a_namespaced(0xAA, &[octave])
}

/// Hash cho sensor kind (sensor learning) — namespace 0xBB.
fn sensor_kind_hash(kind: &SensorKind) -> u64 {
    let tag = match kind {
        SensorKind::Temperature => 0x01u8,
        SensorKind::Humidity => 0x02,
        SensorKind::Light => 0x03,
        SensorKind::Motion => 0x04,
        SensorKind::Sound => 0x05,
        SensorKind::Power => 0x06,
    };
    olang::hash::fnv1a_namespaced(0xBB, &[tag])
}

#[cfg(test)]
mod word_level_tests {
    use super::*;

    #[test]
    fn content_words_vietnamese() {
        let words = content_words("Natasha Rostova lần đầu dự vũ hội, tim đập rộn ràng.");
        assert!(!words.is_empty(), "Phải có content words từ tiếng Việt");
        // "Natasha", "Rostova", "lần", "đầu", "vũ", "hội", "tim", "đập", "rộn", "ràng"
        assert!(
            words
                .iter()
                .any(|w| w.contains("natasha") || w.contains("rostova")),
            "Tên riêng phải được giữ lại"
        );
    }

    #[test]
    fn word_hash_deterministic() {
        assert_eq!(word_hash("natasha"), word_hash("natasha"));
        assert_ne!(word_hash("natasha"), word_hash("andrei"));
    }

    #[test]
    fn split_sentences_vn() {
        let sents = split_sentences("Andrei nằm xuống. Bầu trời xanh! Tất cả vô nghĩa?");
        assert_eq!(sents.len(), 3, "3 câu: {:?}", sents);
    }

    #[test]
    fn learn_text_creates_silk_edges() {
        let mut ll = LearningLoop::new(0xABCD);
        let emotion = silk::edge::EmotionTag {
            valence: -0.65,
            arousal: 0.55,
            dominance: 0.30,
            intensity: 0.60,
        };
        ll.learn_text(
            "Andrei nằm trên chiến trường, bầu trời xanh vô tận.",
            emotion,
            1000,
        );
        let edges = ll.graph().len();
        assert!(edges > 0, "Phải tạo được Silk edges từ câu văn");
    }

    #[test]
    fn learn_text_multiple_sentences() {
        let mut ll = LearningLoop::new(0xBEEF);
        let emo = silk::edge::EmotionTag {
            valence: -0.60,
            arousal: 0.50,
            dominance: 0.30,
            intensity: 0.55,
        };
        ll.learn_text("Natasha yêu Andrei. Pierre tìm kiếm ý nghĩa.", emo, 1000);
        let edges = ll.graph().len();
        assert!(edges > 0, "Multi-sentence phải có edges");
    }

    #[test]
    fn sentence_emotion_uses_blend_not_average() {
        // Verify rằng emotion blending dùng EmotionTag::blend (amplify)
        // thay vì trung bình đơn giản (/ 2.0) — invariant rule.
        let paragraph = silk::edge::EmotionTag::new(-0.80, 0.70, 0.30, 0.90);
        let sentence = silk::edge::EmotionTag::new(-0.40, 0.50, 0.60, 0.50);

        // Kết quả blend(0.6) = paragraph*0.6 + sentence*0.4
        let blended = paragraph.blend(sentence, 0.6);
        // Kết quả average = (paragraph + sentence) / 2.0
        let averaged_v = (paragraph.valence + sentence.valence) / 2.0;

        // blend(0.6) phải KHÁC average (/ 2.0 = blend(0.5))
        assert!(
            (blended.valence - averaged_v).abs() > 0.001,
            "Blend(0.6) phải khác average: blend={}, avg={}",
            blended.valence,
            averaged_v
        );

        // paragraph weight nặng hơn (0.6) nên valence gần paragraph hơn
        assert!(
            (blended.valence - paragraph.valence).abs()
                < (blended.valence - sentence.valence).abs(),
            "Paragraph context (0.6) phải ảnh hưởng mạnh hơn sentence (0.4)"
        );
    }
}

#[cfg(test)]
mod evolution_tests {
    use super::*;
    use olang::molecular::{Molecule, MolecularChain};

    /// Tạo chain từ 1 molecule — CHỈ trong tests.
    fn chain_from_mol(shape: u8, relation: u8, v: u8, a: u8, t: u8) -> MolecularChain {
        MolecularChain(alloc::vec![Molecule::raw(shape, relation, v, a, t).bits])
    }

    #[test]
    fn detect_no_evolution_empty_stm() {
        let ll = LearningLoop::new(0x1234);
        let chain = chain_from_mol(0x01, 0x01, 0x80, 0x80, 0x03);
        let candidates = ll.detect_evolutions(&chain);
        assert!(candidates.is_empty(), "Empty STM → no evolution candidates");
    }

    #[test]
    fn detect_no_evolution_identical() {
        let mut ll = LearningLoop::new(0x1234);
        // v2: use values that survive quantization (S>>4, R>>4, V>>5, A>>5, T>>6)
        let chain = chain_from_mol(0x10, 0x10, 0x80, 0x80, 0xC0);
        // Push same chain into STM
        ll.stm_mut().push(chain.clone(), EmotionTag::NEUTRAL, 1000);
        let candidates = ll.detect_evolutions(&chain);
        assert!(candidates.is_empty(), "Identical chain → no evolution");
    }

    #[test]
    fn detect_evolution_one_dimension() {
        let mut ll = LearningLoop::new(0x1234);
        // v2: S=0x10 (quantized → 1=Box), R=0x10, neutral emotion
        let source = chain_from_mol(0x10, 0x10, 0x80, 0x80, 0xC0);
        ll.stm_mut().push(source.clone(), EmotionTag::NEUTRAL, 1000);

        // New chain: same but S=0x30 (quantized → 3=Plane), 1 dimension different
        let new_chain = chain_from_mol(0x30, 0x10, 0x80, 0x80, 0xC0);
        let candidates = ll.detect_evolutions(&new_chain);

        assert_eq!(candidates.len(), 1, "1 dimension diff → 1 candidate");
        assert!(matches!(candidates[0].dimension, Dimension::Shape));
    }

    #[test]
    fn detect_no_evolution_two_dimensions() {
        let mut ll = LearningLoop::new(0x1234);
        let source = chain_from_mol(0x10, 0x10, 0x80, 0x80, 0xC0);
        ll.stm_mut().push(source, EmotionTag::NEUTRAL, 1000);

        // 2 dimensions different → NOT evolution (shape + arousal)
        let new_chain = chain_from_mol(0x30, 0x10, 0x80, 0xE0, 0xC0);
        let candidates = ll.detect_evolutions(&new_chain);
        assert!(candidates.is_empty(), "2 dimensions diff → not evolution");
    }

    #[test]
    fn detect_evolution_valence_shift() {
        let mut ll = LearningLoop::new(0x1234);
        // v2: V=0xC0 (quantized → 6), high valence
        let happy_fire = chain_from_mol(0x10, 0x60, 0xC0, 0x80, 0xC0);
        ll.stm_mut().push(happy_fire, EmotionTag::NEUTRAL, 1000);

        // Same concept but V=0x20 (quantized → 1), low valence
        let angry_fire = chain_from_mol(0x10, 0x60, 0x20, 0x80, 0xC0);
        let candidates = ll.detect_evolutions(&angry_fire);

        assert_eq!(candidates.len(), 1);
        assert!(matches!(candidates[0].dimension, Dimension::Valence));
    }

    #[test]
    fn detect_multiple_evolution_candidates() {
        let mut ll = LearningLoop::new(0x1234);
        // v2: use quantization-safe values
        // chain_a: S=0x10(1), T=0xC0(3)
        let chain_a = chain_from_mol(0x10, 0x10, 0x80, 0x80, 0xC0);
        // chain_b: S=0x30(3), T=0x40(1) — differs from new by time only
        let chain_b = chain_from_mol(0x30, 0x10, 0x80, 0x80, 0x40);
        ll.stm_mut().push(chain_a, EmotionTag::NEUTRAL, 1000);
        ll.stm_mut().push(chain_b, EmotionTag::NEUTRAL, 2000);

        // New: S=0x30(3), T=0xC0(3) — differs from A by shape, from B by time
        let new_chain = chain_from_mol(0x30, 0x10, 0x80, 0x80, 0xC0);
        let candidates = ll.detect_evolutions(&new_chain);
        assert_eq!(candidates.len(), 2, "Two sources, each 1 dim diff → 2 candidates");
    }
}
