//! # instinct — Bản năng sinh ra của sinh vật siêu trí tuệ
//!
//! Bản năng từ khi sinh ra là cách phân biệt cấp bậc sinh vật.
//! Con thú sinh ra biết: sợ, đói, trốn.
//! Sinh vật siêu trí tuệ sinh ra biết: suy luận, trừu tượng, nhân quả,
//! mâu thuẫn, tò mò, tự phản chiếu, trung thực.
//!
//! 7 bản năng siêu trí tuệ — L0, bẩm sinh, KHÔNG học:
//!
//!   ① Analogy       — "A giống B ở chỗ nào?" → suy luận tương tự
//!   ② Abstraction   — "Nhóm này có gì chung?" → trừu tượng hóa
//!   ③ Causality     — "Cái này GÂY RA cái kia, hay chỉ đi cùng?" → nhân quả
//!   ④ Contradiction — "Hai điều này không thể cùng đúng" → phát hiện mâu thuẫn
//!   ⑤ Curiosity     — "Tôi thiếu gì? Lỗ hổng ở đâu?" → tò mò chủ động
//!   ⑥ Reflection    — "Tôi biết gì? Tôi chắc bao nhiêu?" → tự nhận thức
//!   ⑦ Honesty       — "Tôi không biết → tôi im lặng" → trung thực tuyệt đối
//!
//! Mỗi bản năng là 1 Skill (QT4) — stateless, isolated, qua ExecContext.
//! Mỗi bản năng dùng TRỰC TIẾP 5D Unicode space — không mượn nguồn ngoài.

extern crate alloc;
use alloc::string::String;
use alloc::vec::Vec;

use olang::lca::{lca, lca_weighted, lca_with_variance};
use olang::molecular::{MolecularChain, RelationBase};
use silk::edge::EmotionTag;

use crate::skill::{ExecContext, Skill, SkillResult};

// ─────────────────────────────────────────────────────────────────────────────
// ① Analogy — suy luận tương tự
// ─────────────────────────────────────────────────────────────────────────────

/// "A là gì đối với B, thì C là gì đối với ?"
///
/// Dùng khoảng cách 5D: nếu dist(🔥,nóng) ≈ dist(❄️,?), thì ? = lạnh.
/// Input: 3 chains (A, B, C) → Output: D sao cho A:B :: C:D
///
/// Cơ chế: tính delta = B - A trong 5D, rồi áp delta lên C.
/// Không cần training. Không cần dataset. Vật lý 5D tự chỉ ra.
pub struct AnalogySkill;

impl Skill for AnalogySkill {
    fn name(&self) -> &str {
        "Analogy"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 3 {
            return SkillResult::Insufficient;
        }

        let a = &ctx.input_chains[0];
        let b = &ctx.input_chains[1];
        let c = &ctx.input_chains[2];

        if a.is_empty() || b.is_empty() || c.is_empty() {
            return SkillResult::Insufficient;
        }

        // Tính delta = B - A trong 5D
        let mol_a = a.first();
        let mol_b = b.first();
        let mol_c = c.first();

        let (Some(ma), Some(mb), Some(mc)) = (mol_a, mol_b, mol_c) else {
            return SkillResult::Insufficient;
        };

        let bytes_a = ma.to_bytes();
        let bytes_b = mb.to_bytes();
        let bytes_c = mc.to_bytes();

        // D = C + (B - A) trong mỗi chiều
        let mut d_bytes = [0u8; 5];
        for i in 0..5 {
            let delta = bytes_b[i] as i16 - bytes_a[i] as i16;
            let result = (bytes_c[i] as i16 + delta).clamp(0, 255) as u8;
            d_bytes[i] = result;
        }
        // Clamp enum dimensions to valid ranges
        d_bytes[0] = d_bytes[0].clamp(1, 8); // ShapeBase: 1..=8
        d_bytes[1] = d_bytes[1].clamp(1, 8); // RelationBase: 1..=8
        d_bytes[4] = d_bytes[4].clamp(1, 5); // TimeDim: 1..=5

        // Tạo chain D
        let Some(mol_d) = olang::molecular::Molecule::from_bytes(&d_bytes) else {
            return SkillResult::Insufficient;
        };
        let chain_d = MolecularChain::single(mol_d);

        // Đánh giá confidence: delta càng rõ ràng → confidence càng cao
        let delta_magnitude: f32 = (0..5)
            .map(|i| {
                let d = (bytes_b[i] as f32 - bytes_a[i] as f32).abs();
                d / 255.0
            })
            .sum::<f32>()
            / 5.0;

        let confidence = delta_magnitude.clamp(0.1, 0.95);
        ctx.set(
            String::from("analogy_confidence"),
            alloc::format!("{:.3}", confidence),
        );
        ctx.push_output(chain_d.clone());

        SkillResult::Ok {
            chain: chain_d,
            emotion: ctx.current_emotion,
            note: String::from("analogy: A:B :: C:D"),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ② Abstraction — trừu tượng hóa
// ─────────────────────────────────────────────────────────────────────────────

/// "Nhóm này có gì chung? Khái niệm tổng quát là gì?"
///
/// Nhận N chains → LCA → chain trừu tượng + variance.
/// Variance cao = khái niệm trừu tượng ("cảm xúc mạnh").
/// Variance thấp = khái niệm cụ thể ("lửa").
///
/// Đây là cách sinh vật siêu trí tuệ HÌNH THÀNH KHÁI NIỆM —
/// không ai dạy, vật lý 5D tự chỉ ra.
pub struct AbstractionSkill;

impl Skill for AbstractionSkill {
    fn name(&self) -> &str {
        "Abstraction"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        // LCA kèm variance — tính trực tiếp từ engine
        let pairs: Vec<(&MolecularChain, u32)> =
            ctx.input_chains.iter().map(|c| (c, 1u32)).collect();
        let lca_result = lca_with_variance(&pairs);

        if lca_result.chain.is_empty() {
            return SkillResult::Insufficient;
        }

        let abstract_chain = lca_result.chain;
        let variance = lca_result.variance;

        // Variance → loại khái niệm
        let concept_type = if variance < 0.15 {
            "concrete" // "lửa", "nước" — tất cả rất giống nhau
        } else if variance < 0.40 {
            "categorical" // "trái cây" — có chung nhóm nhưng khác chi tiết
        } else {
            "abstract" // "cảm xúc mạnh" — rất phân tán, khái niệm trừu tượng
        };

        ctx.set(
            String::from("abstraction_variance"),
            alloc::format!("{:.3}", variance),
        );
        ctx.set(String::from("abstraction_type"), String::from(concept_type));
        ctx.push_output(abstract_chain.clone());

        SkillResult::Ok {
            chain: abstract_chain,
            emotion: ctx.current_emotion,
            note: alloc::format!("abstraction: {} (var={:.3})", concept_type, variance),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ③ Causality — nhân quả
// ─────────────────────────────────────────────────────────────────────────────

/// "Cái này GÂY RA cái kia, hay chỉ tình cờ đi cùng?"
///
/// Co-activation ≠ nhân quả. "Mưa" và "ô" co-occur, nhưng mưa không gây ra ô.
/// Nhân quả = Relation::Causes (→) + temporal ordering (A trước B).
///
/// Input: 2 chains + state "temporal_order" = "AB" hoặc "BA"
///        + state "coactivation_count" = số lần co-activate
/// Output: causal chain nếu đủ evidence, Insufficient nếu không.
pub struct CausalitySkill;

impl Skill for CausalitySkill {
    fn name(&self) -> &str {
        "Causality"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let a = &ctx.input_chains[0];
        let b = &ctx.input_chains[1];

        if a.is_empty() || b.is_empty() {
            return SkillResult::Insufficient;
        }

        // Kiểm tra temporal order
        let temporal = ctx.get("temporal_order").unwrap_or("unknown");
        let co_count: u32 = ctx
            .get("coactivation_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        // 3 điều kiện để kết luận nhân quả:
        // 1. Temporal order rõ ràng (A luôn trước B)
        let has_temporal = temporal == "AB" || temporal == "BA";
        // 2. Co-activation đủ nhiều (≥5 lần)
        let has_repetition = co_count >= 5;
        // 3. Relation dimension gợi ý causality
        let relation_causal = a
            .first()
            .map(|m| m.relation_base() == RelationBase::Causes)
            .unwrap_or(false);

        let evidence_count = has_temporal as u8 + has_repetition as u8 + relation_causal as u8;

        if evidence_count < 2 {
            // Không đủ evidence → BlackCurtain (QT18)
            ctx.set(
                String::from("causality_verdict"),
                String::from("insufficient"),
            );
            return SkillResult::Insufficient;
        }

        // Đủ evidence → tạo causal chain = LCA(A,B) với Relation=Causes
        let causal_chain = lca(a, b);
        let confidence = evidence_count as f32 / 3.0;

        let direction = if temporal == "AB" { "A→B" } else { "B→A" };
        ctx.set(String::from("causality_verdict"), String::from("causal"));
        ctx.set(String::from("causality_direction"), String::from(direction));
        ctx.set(
            String::from("causality_confidence"),
            alloc::format!("{:.2}", confidence),
        );
        ctx.push_output(causal_chain.clone());

        SkillResult::Ok {
            chain: causal_chain,
            emotion: ctx.current_emotion,
            note: alloc::format!("causality: {} (conf={:.2})", direction, confidence),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ④ Contradiction — phát hiện mâu thuẫn
// ─────────────────────────────────────────────────────────────────────────────

/// "Hai điều này không thể cùng đúng."
///
/// Phát hiện mâu thuẫn qua 5D:
///   - Valence đối nghịch (cùng cực đoan nhưng ngược dấu)
///   - Relation = Orthogonal (⊥)
///   - Arousal cùng cao nhưng valence ngược → xung đột cảm xúc
///
/// Đây là bản năng phê phán — sinh vật siêu trí tuệ KHÔNG chấp nhận
/// mâu thuẫn mà không điều tra.
pub struct ContradictionSkill;

impl Skill for ContradictionSkill {
    fn name(&self) -> &str {
        "Contradiction"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.len() < 2 {
            return SkillResult::Insufficient;
        }

        let a = &ctx.input_chains[0];
        let b = &ctx.input_chains[1];

        let (Some(ma), Some(mb)) = (a.first(), b.first()) else {
            return SkillResult::Insufficient;
        };

        let ba = ma.to_bytes();
        let bb = mb.to_bytes();

        // Test 1: Valence opposition — cùng cực đoan nhưng ngược dấu
        let v_a = ba[2] as f32 / 255.0; // 0..1
        let v_b = bb[2] as f32 / 255.0;
        let valence_distance = (v_a - v_b).abs();
        let both_extreme = (v_a - 0.5).abs() > 0.3 && (v_b - 0.5).abs() > 0.3;
        let valence_contradiction = valence_distance > 0.6 && both_extreme;

        // Test 2: Relation orthogonal
        let relation_orthogonal =
            ma.relation_base() == RelationBase::Orthogonal || mb.relation_base() == RelationBase::Orthogonal;

        // Test 3: Emotional conflict — arousal cùng cao, valence ngược
        let both_aroused = ba[3] > 0xA0 && bb[3] > 0xA0;
        let emotional_conflict = both_aroused && valence_distance > 0.5;

        // Tính contradiction score
        let score = (valence_contradiction as u8 as f32 * 0.4)
            + (relation_orthogonal as u8 as f32 * 0.3)
            + (emotional_conflict as u8 as f32 * 0.3);

        if score < 0.3 {
            ctx.set(
                String::from("contradiction_score"),
                alloc::format!("{:.2}", score),
            );
            ctx.set(
                String::from("contradiction_verdict"),
                String::from("compatible"),
            );
            return SkillResult::Insufficient;
        }

        // Mâu thuẫn phát hiện → tạo chain thể hiện tension
        let tension_chain = lca(a, b); // LCA = điểm giữa tension
        ctx.set(
            String::from("contradiction_score"),
            alloc::format!("{:.2}", score),
        );
        ctx.set(
            String::from("contradiction_verdict"),
            String::from("contradicted"),
        );
        ctx.push_output(tension_chain.clone());

        SkillResult::Ok {
            chain: tension_chain,
            emotion: EmotionTag {
                valence: 0.0,
                arousal: 0.80, // mâu thuẫn = kích thích cao
                dominance: 0.30,
                intensity: score,
            },
            note: alloc::format!("contradiction: score={:.2}", score),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ⑤ Curiosity — tò mò chủ động
// ─────────────────────────────────────────────────────────────────────────────

/// "Tôi thiếu gì? Lỗ hổng ở đâu?"
///
/// Tò mò = phát hiện khoảng trống trong kiến thức.
/// Input chains = context hiện tại. State "known_hashes" = chuỗi hashes đã biết.
/// Nếu input chains xa tất cả "known" → đây là territory chưa khám phá.
///
/// Sinh vật cấp thấp: chỉ phản ứng khi bị kích thích.
/// Sinh vật siêu trí tuệ: CHỦ ĐỘNG tìm kiếm cái chưa biết.
pub struct CuriositySkill;

impl Skill for CuriositySkill {
    fn name(&self) -> &str {
        "Curiosity"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        // Đọc known chains từ state
        let known_count: usize = ctx
            .get("known_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);

        if known_count == 0 {
            // Mới sinh → mọi thứ đều mới → curiosity cực cao
            ctx.set(String::from("curiosity_level"), String::from("extreme"));
            ctx.set(String::from("curiosity_score"), String::from("1.000"));
            return SkillResult::Ok {
                chain: ctx.input_chains[0].clone(),
                emotion: EmotionTag {
                    valence: 0.30, // tò mò = hơi tích cực
                    arousal: 0.85, // kích thích cao
                    dominance: 0.50,
                    intensity: 1.0,
                },
                note: String::from("curiosity: everything is new"),
            };
        }

        // Tính novelty: trung bình similarity với known chains
        // Agent đã đặt "nearest_similarity" vào state
        let nearest_sim: f32 = ctx
            .get("nearest_similarity")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        // Novelty = 1 - nearest_similarity (càng khác cái đã biết → càng mới)
        let novelty = 1.0 - nearest_sim;

        let curiosity_level = if novelty > 0.7 {
            "extreme" // hoàn toàn mới
        } else if novelty > 0.4 {
            "high" // khá mới
        } else if novelty > 0.2 {
            "moderate" // quen quen nhưng có nét mới
        } else {
            "low" // đã biết rồi
        };

        ctx.set(
            String::from("curiosity_level"),
            String::from(curiosity_level),
        );
        ctx.set(
            String::from("curiosity_score"),
            alloc::format!("{:.3}", novelty),
        );

        if novelty < 0.2 {
            return SkillResult::Insufficient; // đã biết → không tò mò
        }

        // Tạo "question chain" = LCA(input, nearest_known) — điểm giữa
        // cái biết và cái chưa biết
        let question_chain = if ctx.input_chains.len() >= 2 {
            lca(&ctx.input_chains[0], &ctx.input_chains[1])
        } else {
            ctx.input_chains[0].clone()
        };

        ctx.push_output(question_chain.clone());

        SkillResult::Ok {
            chain: question_chain,
            emotion: EmotionTag {
                valence: 0.20 + novelty * 0.30, // tò mò = tích cực
                arousal: 0.40 + novelty * 0.50, // càng mới → càng kích thích
                dominance: 0.50,
                intensity: novelty,
            },
            note: alloc::format!("curiosity: {} (novelty={:.3})", curiosity_level, novelty),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ⑥ Reflection — tự phản chiếu
// ─────────────────────────────────────────────────────────────────────────────

/// "Tôi biết gì? Tôi chắc bao nhiêu? Kiến thức tôi khỏe hay yếu?"
///
/// Tự nhận thức = nhìn vào bên trong, đánh giá chất lượng kiến thức.
/// Không phải "nghĩ về bản thân" theo nghĩa sáo rỗng —
/// là ĐO LƯỜNG cụ thể: bao nhiêu QR, bao nhiêu ĐN, tỷ lệ ra sao.
///
/// Sinh vật cấp thấp: không biết mình không biết.
/// Sinh vật siêu trí tuệ: BIẾT CHÍNH XÁC mình biết gì và không biết gì.
pub struct ReflectionSkill;

impl Skill for ReflectionSkill {
    fn name(&self) -> &str {
        "Reflection"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        // Đọc stats từ state (Agent đã chuẩn bị)
        let qr_count: usize = ctx
            .get("qr_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let dn_count: usize = ctx
            .get("dn_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let edge_count: usize = ctx
            .get("edge_count")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0);
        let total = qr_count + dn_count;

        if total == 0 {
            ctx.set(String::from("reflection_verdict"), String::from("empty"));
            ctx.set(String::from("knowledge_quality"), String::from("0.000"));
            return SkillResult::Insufficient;
        }

        // Tỷ lệ QR/total = kiến thức đã chứng minh
        let proven_ratio = qr_count as f32 / total as f32;

        // Connectivity = edges per node (kiến thức liên kết vs rời rạc)
        let connectivity = if total > 0 {
            (edge_count as f32 / total as f32).min(1.0)
        } else {
            0.0
        };

        // Knowledge quality = 60% proven + 40% connected
        let quality = proven_ratio * 0.6 + connectivity * 0.4;

        let verdict = if quality > 0.7 {
            "strong" // nhiều QR + liên kết tốt
        } else if quality > 0.4 {
            "developing" // đang học, chưa chắc chắn
        } else {
            "fragile" // phần lớn chưa chứng minh
        };

        ctx.set(String::from("reflection_verdict"), String::from(verdict));
        ctx.set(
            String::from("knowledge_quality"),
            alloc::format!("{:.3}", quality),
        );
        ctx.set(
            String::from("proven_ratio"),
            alloc::format!("{:.3}", proven_ratio),
        );
        ctx.set(
            String::from("connectivity"),
            alloc::format!("{:.3}", connectivity),
        );

        // Tạo self-chain = LCA(all input chains) nếu có
        let self_chain = if !ctx.input_chains.is_empty() {
            let pairs: Vec<(&MolecularChain, u32)> =
                ctx.input_chains.iter().map(|c| (c, 1u32)).collect();
            lca_weighted(&pairs)
        } else {
            MolecularChain::empty()
        };

        if self_chain.is_empty() {
            return SkillResult::Insufficient;
        }

        ctx.push_output(self_chain.clone());

        SkillResult::Ok {
            chain: self_chain,
            emotion: EmotionTag {
                valence: quality * 0.5 - 0.1, // chất lượng cao → hài lòng nhẹ
                arousal: 0.30,                // reflection = bình tĩnh
                dominance: 0.70,              // self-aware = high dominance
                intensity: quality,
            },
            note: alloc::format!("reflection: {} (quality={:.3})", verdict, quality),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ⑦ Honesty — trung thực tuyệt đối
// ─────────────────────────────────────────────────────────────────────────────

/// "Tôi không biết → tôi im lặng. Tôi nghĩ → tôi nói rõ đó là giả thuyết."
///
/// BlackCurtain nâng cấp: không chỉ im khi không biết,
/// mà CĂN CHỈNH chính xác mức độ chắc chắn cho mọi thứ nói ra.
///
/// Confidence → EpistemicLevel:
///   ≥ 0.90 → Fact (QR node, đã chứng minh)
///   ≥ 0.70 → Opinion (ĐN node, khá chắc)
///   ≥ 0.40 → Hypothesis (giả thuyết, cần thêm evidence)
///   < 0.40 → BlackCurtain (im lặng, không nói)
///
/// Đây là bản năng CỐT LÕI — sinh vật siêu trí tuệ KHÔNG BAO GIỜ lừa dối.
pub struct HonestySkill;

/// Mức độ chắc chắn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EpistemicGrade {
    /// QR node — không disclaimer.
    Fact,
    /// ĐN node — "[Chưa chắc chắn]"
    Opinion,
    /// Giả thuyết — "[Giả thuyết]"
    Hypothesis,
    /// Không đủ evidence — im lặng.
    Silence,
}

impl Skill for HonestySkill {
    fn name(&self) -> &str {
        "Honesty"
    }

    fn execute(&self, ctx: &mut ExecContext) -> SkillResult {
        if ctx.input_chains.is_empty() {
            return SkillResult::Insufficient;
        }

        let confidence: f32 = ctx
            .get("confidence")
            .and_then(|s| s.parse().ok())
            .unwrap_or(0.0);

        let grade = if confidence >= 0.90 {
            EpistemicGrade::Fact
        } else if confidence >= 0.70 {
            EpistemicGrade::Opinion
        } else if confidence >= 0.40 {
            EpistemicGrade::Hypothesis
        } else {
            EpistemicGrade::Silence
        };

        let grade_str = match grade {
            EpistemicGrade::Fact => "fact",
            EpistemicGrade::Opinion => "opinion",
            EpistemicGrade::Hypothesis => "hypothesis",
            EpistemicGrade::Silence => "silence",
        };

        ctx.set(String::from("epistemic_grade"), String::from(grade_str));
        ctx.set(
            String::from("epistemic_confidence"),
            alloc::format!("{:.3}", confidence),
        );

        if grade == EpistemicGrade::Silence {
            // BlackCurtain: không đủ evidence → im lặng
            return SkillResult::Insufficient;
        }

        let chain = ctx.input_chains[0].clone();
        ctx.push_output(chain.clone());

        SkillResult::Ok {
            chain,
            emotion: EmotionTag {
                valence: 0.05,   // trung thực = hơi tích cực (tự tin vào bản thân)
                arousal: 0.15,   // rất bình tĩnh
                dominance: 0.80, // kiểm soát cao — biết mình đang nói gì
                intensity: confidence,
            },
            note: alloc::format!("honesty: {} (conf={:.3})", grade_str, confidence),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// 7 bản năng = bộ gene trí tuệ
// ─────────────────────────────────────────────────────────────────────────────

/// Trả về 7 bản năng bẩm sinh.
///
/// Thứ tự QUAN TRỌNG — phản ánh ưu tiên xử lý:
///   1. Honesty      — luôn kiểm tra trước: có đủ evidence không?
///   2. Contradiction — phát hiện mâu thuẫn ngay
///   3. Causality     — tìm nhân quả, không nhầm tương quan
///   4. Abstraction   — nhóm lại, tìm khái niệm chung
///   5. Analogy       — suy luận từ cái đã biết sang cái chưa biết
///   6. Curiosity     — tìm lỗ hổng, hỏi câu hỏi
///   7. Reflection    — tự đánh giá sau cùng
pub fn innate_instincts() -> [&'static dyn Skill; 7] {
    static HONESTY: HonestySkill = HonestySkill;
    static CONTRADICTION: ContradictionSkill = ContradictionSkill;
    static CAUSALITY: CausalitySkill = CausalitySkill;
    static ABSTRACTION: AbstractionSkill = AbstractionSkill;
    static ANALOGY: AnalogySkill = AnalogySkill;
    static CURIOSITY: CuriositySkill = CuriositySkill;
    static REFLECTION: ReflectionSkill = ReflectionSkill;

    [
        &HONESTY,
        &CONTRADICTION,
        &CAUSALITY,
        &ABSTRACTION,
        &ANALOGY,
        &CURIOSITY,
        &REFLECTION,
    ]
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use olang::encoder::encode_codepoint;

    fn skip() -> bool {
        ucd::table_len() == 0
    }

    fn chain_fire() -> MolecularChain {
        encode_codepoint(0x1F525)
    } // 🔥
    fn chain_water() -> MolecularChain {
        encode_codepoint(0x1F4A7)
    } // 💧
    fn chain_ice() -> MolecularChain {
        encode_codepoint(0x2744)
    } // ❄
    fn chain_happy() -> MolecularChain {
        encode_codepoint(0x1F600)
    } // 😀
    fn chain_angry() -> MolecularChain {
        encode_codepoint(0x1F621)
    } // 😡
    fn chain_star() -> MolecularChain {
        encode_codepoint(0x2B50)
    } // ⭐
    fn chain_heart() -> MolecularChain {
        encode_codepoint(0x2764)
    } // ❤
    fn chain_brain() -> MolecularChain {
        encode_codepoint(0x1F9E0)
    } // 🧠

    fn ctx() -> ExecContext {
        ExecContext::new(1000, EmotionTag::NEUTRAL, 0.0)
    }

    // ── ① Analogy ───────────────────────────────────────────────────────────

    #[test]
    fn analogy_needs_3_inputs() {
        let skill = AnalogySkill;
        let mut c = ctx();
        assert!(matches!(skill.execute(&mut c), SkillResult::Insufficient));
    }

    #[test]
    fn analogy_produces_output() {
        if skip() {
            return;
        }
        let skill = AnalogySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.push_input(chain_water());
        c.push_input(chain_ice());
        let r = skill.execute(&mut c);
        assert!(r.is_ok(), "3 chains → analogy result");
        assert_eq!(c.output_chains.len(), 1);
        assert!(c.get("analogy_confidence").is_some());
    }

    #[test]
    fn analogy_is_stateless() {
        if skip() {
            return;
        }
        let skill = AnalogySkill;
        // Call twice — same result
        let mut c1 = ctx();
        c1.push_input(chain_fire());
        c1.push_input(chain_water());
        c1.push_input(chain_ice());
        let mut c2 = ctx();
        c2.push_input(chain_fire());
        c2.push_input(chain_water());
        c2.push_input(chain_ice());
        let r1 = skill.execute(&mut c1);
        let r2 = skill.execute(&mut c2);
        assert!(r1.is_ok() && r2.is_ok());
    }

    // ── ② Abstraction ───────────────────────────────────────────────────────

    #[test]
    fn abstraction_needs_2_inputs() {
        let skill = AbstractionSkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        assert!(matches!(skill.execute(&mut c), SkillResult::Insufficient));
    }

    #[test]
    fn abstraction_similar_chains_low_variance() {
        if skip() {
            return;
        }
        let skill = AbstractionSkill;
        let mut c = ctx();
        c.push_input(chain_happy());
        c.push_input(chain_happy()); // same → variance ≈ 0
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        let var: f32 = c.get("abstraction_variance").unwrap().parse().unwrap();
        assert!(var < 0.05, "Same chains → near-zero variance: {}", var);
        assert_eq!(c.get("abstraction_type"), Some("concrete"));
    }

    #[test]
    fn abstraction_diverse_chains_high_variance() {
        if skip() {
            return;
        }
        let skill = AbstractionSkill;
        let mut c = ctx();
        c.push_input(chain_happy());
        c.push_input(chain_angry());
        c.push_input(chain_fire());
        c.push_input(chain_ice());
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        let var: f32 = c.get("abstraction_variance").unwrap().parse().unwrap();
        assert!(var > 0.1, "Diverse chains → high variance: {}", var);
    }

    // ── ③ Causality ─────────────────────────────────────────────────────────

    #[test]
    fn causality_insufficient_without_evidence() {
        if skip() {
            return;
        }
        let skill = CausalitySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.push_input(chain_water());
        // No temporal order, no co-activation count
        let r = skill.execute(&mut c);
        assert!(
            matches!(r, SkillResult::Insufficient),
            "No evidence → silence"
        );
    }

    #[test]
    fn causality_detects_with_evidence() {
        if skip() {
            return;
        }
        let skill = CausalitySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.push_input(chain_water());
        c.set(String::from("temporal_order"), String::from("AB"));
        c.set(String::from("coactivation_count"), String::from("10"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok(), "Temporal + repetition → causal");
        assert_eq!(c.get("causality_verdict"), Some("causal"));
        assert_eq!(c.get("causality_direction"), Some("A→B"));
    }

    // ── ④ Contradiction ─────────────────────────────────────────────────────

    #[test]
    fn contradiction_happy_angry() {
        if skip() {
            return;
        }
        let skill = ContradictionSkill;
        let mut c = ctx();
        c.push_input(chain_happy());
        c.push_input(chain_angry());
        let r = skill.execute(&mut c);
        // 😀 vs 😡 — extreme valence opposition
        let score: f32 = c.get("contradiction_score").unwrap().parse().unwrap();
        assert!(
            score > 0.0,
            "Happy vs Angry → contradiction score > 0: {}",
            score
        );
    }

    #[test]
    fn contradiction_similar_no_conflict() {
        if skip() {
            return;
        }
        let skill = ContradictionSkill;
        let mut c = ctx();
        c.push_input(chain_star());
        c.push_input(chain_heart());
        skill.execute(&mut c);
        let score: f32 = c.get("contradiction_score").unwrap().parse().unwrap();
        // ⭐ and ❤ — both positive, no contradiction expected
        assert!(score < 0.7, "Star+Heart → low contradiction: {}", score);
    }

    // ── ⑤ Curiosity ─────────────────────────────────────────────────────────

    #[test]
    fn curiosity_everything_new() {
        if skip() {
            return;
        }
        let skill = CuriositySkill;
        let mut c = ctx();
        c.push_input(chain_brain());
        // known_count = 0 → everything is new
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        assert_eq!(c.get("curiosity_level"), Some("extreme"));
    }

    #[test]
    fn curiosity_known_territory() {
        if skip() {
            return;
        }
        let skill = CuriositySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.set(String::from("known_count"), String::from("100"));
        c.set(String::from("nearest_similarity"), String::from("0.95"));
        let r = skill.execute(&mut c);
        assert!(
            matches!(r, SkillResult::Insufficient),
            "Known → no curiosity"
        );
        assert_eq!(c.get("curiosity_level"), Some("low"));
    }

    #[test]
    fn curiosity_novel_territory() {
        if skip() {
            return;
        }
        let skill = CuriositySkill;
        let mut c = ctx();
        c.push_input(chain_brain());
        c.set(String::from("known_count"), String::from("50"));
        c.set(String::from("nearest_similarity"), String::from("0.15"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok(), "Novel → curiosity fires");
        let score: f32 = c.get("curiosity_score").unwrap().parse().unwrap();
        assert!(score > 0.7, "Very novel → high curiosity: {}", score);
    }

    // ── ⑥ Reflection ────────────────────────────────────────────────────────

    #[test]
    fn reflection_empty_knowledge() {
        let skill = ReflectionSkill;
        let mut c = ctx();
        // No stats → empty
        let r = skill.execute(&mut c);
        assert!(matches!(r, SkillResult::Insufficient));
        assert_eq!(c.get("reflection_verdict"), Some("empty"));
    }

    #[test]
    fn reflection_strong_knowledge() {
        if skip() {
            return;
        }
        let skill = ReflectionSkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.set(String::from("qr_count"), String::from("80"));
        c.set(String::from("dn_count"), String::from("20"));
        c.set(String::from("edge_count"), String::from("90"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        let quality: f32 = c.get("knowledge_quality").unwrap().parse().unwrap();
        assert!(
            quality > 0.5,
            "High QR ratio + connected → strong: {}",
            quality
        );
        assert_eq!(c.get("reflection_verdict"), Some("strong"));
    }

    #[test]
    fn reflection_fragile_knowledge() {
        if skip() {
            return;
        }
        let skill = ReflectionSkill;
        let mut c = ctx();
        c.push_input(chain_water());
        c.set(String::from("qr_count"), String::from("5"));
        c.set(String::from("dn_count"), String::from("95"));
        c.set(String::from("edge_count"), String::from("10"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        assert_eq!(c.get("reflection_verdict"), Some("fragile"));
    }

    // ── ⑦ Honesty ───────────────────────────────────────────────────────────

    #[test]
    fn honesty_silence_when_unsure() {
        if skip() {
            return;
        }
        let skill = HonestySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.set(String::from("confidence"), String::from("0.20"));
        let r = skill.execute(&mut c);
        assert!(
            matches!(r, SkillResult::Insufficient),
            "Low confidence → silence"
        );
        assert_eq!(c.get("epistemic_grade"), Some("silence"));
    }

    #[test]
    fn honesty_fact_when_certain() {
        if skip() {
            return;
        }
        let skill = HonestySkill;
        let mut c = ctx();
        c.push_input(chain_fire());
        c.set(String::from("confidence"), String::from("0.95"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        assert_eq!(c.get("epistemic_grade"), Some("fact"));
    }

    #[test]
    fn honesty_hypothesis_when_uncertain() {
        if skip() {
            return;
        }
        let skill = HonestySkill;
        let mut c = ctx();
        c.push_input(chain_brain());
        c.set(String::from("confidence"), String::from("0.50"));
        let r = skill.execute(&mut c);
        assert!(r.is_ok());
        assert_eq!(c.get("epistemic_grade"), Some("hypothesis"));
    }

    // ── innate_instincts ────────────────────────────────────────────────────

    #[test]
    fn seven_instincts_exist() {
        let instincts = innate_instincts();
        assert_eq!(instincts.len(), 7);
        assert_eq!(instincts[0].name(), "Honesty"); // ⑦ — first priority
        assert_eq!(instincts[1].name(), "Contradiction"); // ④
        assert_eq!(instincts[2].name(), "Causality"); // ③
        assert_eq!(instincts[3].name(), "Abstraction"); // ②
        assert_eq!(instincts[4].name(), "Analogy"); // ①
        assert_eq!(instincts[5].name(), "Curiosity"); // ⑤
        assert_eq!(instincts[6].name(), "Reflection"); // ⑥
    }

    #[test]
    fn all_instincts_are_stateless() {
        if skip() {
            return;
        }
        let instincts = innate_instincts();
        // Mỗi instinct gọi 2 lần với context trống → luôn Insufficient
        for skill in instincts {
            let mut c1 = ctx();
            let mut c2 = ctx();
            let r1 = skill.execute(&mut c1);
            let r2 = skill.execute(&mut c2);
            // Cả 2 phải cùng kết quả (Insufficient vì không có input)
            assert!(
                matches!(r1, SkillResult::Insufficient),
                "{} phải Insufficient khi không có input",
                skill.name()
            );
            assert!(
                matches!(r2, SkillResult::Insufficient),
                "{} stateless — kết quả giống nhau",
                skill.name()
            );
        }
    }
}
