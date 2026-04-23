//! Decide whether a new JD is similar enough to a prior one to reuse the
//! existing tailored resume.  Simple TF-IDF-ish cosine on token bags: good
//! enough for MVP and needs no model download.

use std::collections::HashMap;

/// Cosine similarity between two JD token bags.  Returns `[0, 1]`.
pub fn similarity(a: &str, b: &str) -> f32 {
    let av = bag(a);
    let bv = bag(b);
    if av.is_empty() || bv.is_empty() {
        return 0.0;
    }
    let mut dot = 0.0f32;
    let mut na = 0.0f32;
    let mut nb = 0.0f32;
    for (_, v) in av.iter() {
        na += (*v as f32).powi(2);
    }
    for (_, v) in bv.iter() {
        nb += (*v as f32).powi(2);
    }
    for (k, va) in av.iter() {
        if let Some(vb) = bv.get(k) {
            dot += (*va as f32) * (*vb as f32);
        }
    }
    dot / (na.sqrt() * nb.sqrt()).max(1e-9)
}

/// Threshold used by the reuse decision.  Callers may take the setting from
/// user config, but 0.85 is a safe default.
pub const DEFAULT_REUSE_THRESHOLD: f32 = 0.85;

#[derive(Debug, Clone, Copy)]
pub enum ReuseDecision {
    Reuse,
    EditedReuse,
    New,
}

pub fn decide(similarity: f32) -> ReuseDecision {
    if similarity >= DEFAULT_REUSE_THRESHOLD {
        ReuseDecision::Reuse
    } else if similarity >= 0.65 {
        ReuseDecision::EditedReuse
    } else {
        ReuseDecision::New
    }
}

fn bag(text: &str) -> HashMap<String, u32> {
    let mut m: HashMap<String, u32> = HashMap::new();
    for tok in text.split(|c: char| !c.is_ascii_alphanumeric()) {
        if tok.len() < 3 {
            continue;
        }
        if STOP.contains(&tok.to_ascii_lowercase().as_str()) {
            continue;
        }
        *m.entry(tok.to_ascii_lowercase()).or_insert(0) += 1;
    }
    m
}

const STOP: &[&str] = &[
    "the", "and", "for", "with", "you", "are", "our", "have", "will", "from",
    "that", "this", "your", "any", "all", "can", "not", "but", "we're", "we",
    "their", "they", "them", "such", "who", "per", "its", "it's", "a",
];
