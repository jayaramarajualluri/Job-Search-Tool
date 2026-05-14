//! Opt-in AI rewriting of resume bullets via Claude.
//!
//! Honesty constraints baked into the prompt:
//!   - Preserve every fact (metrics, dates, company names, tech, project names).
//!   - Never add experiences, skills, or accomplishments not in the original.
//!   - Only change wording / emphasis to align with the JD's terminology.
//!   - Keep bullet lengths similar.
//!
//! The bullet count is required to match the input exactly — if Claude
//! returns a different count we treat it as a failure and fall back to
//! the rule-based bullets, never silently dropping content.

use crate::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};

const ANTHROPIC_API: &str = "https://api.anthropic.com/v1/messages";
const ANTHROPIC_VERSION: &str = "2023-06-01";

/// Rewrite a flat list of bullets to align with a JD's diction.  The
/// returned vec has the same length as `bullets` in the same order.
pub async fn rewrite_bullets(
    api_key: &str,
    model: &str,
    bullets: &[String],
    jd_skills: &[String],
    jd_excerpt: &str,
) -> AppResult<Vec<String>> {
    if bullets.is_empty() {
        return Ok(vec![]);
    }

    let prompt = build_prompt(bullets, jd_skills, jd_excerpt);

    let req = AnthropicRequest {
        model: model.to_string(),
        max_tokens: 2048,
        messages: vec![Msg {
            role: "user",
            content: prompt,
        }],
    };

    let client = crate::ingestion::http::client()?;
    let resp = client
        .post(ANTHROPIC_API)
        .header("x-api-key", api_key)
        .header("anthropic-version", ANTHROPIC_VERSION)
        .header("content-type", "application/json")
        .json(&req)
        .send()
        .await?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(AppError::Other(format!(
            "Anthropic API returned {}: {}",
            status,
            truncate(&body, 240)
        )));
    }

    let parsed: AnthropicResponse = resp.json().await?;
    let text = parsed
        .content
        .into_iter()
        .find(|c| c.r#type == "text")
        .map(|c| c.text)
        .ok_or_else(|| AppError::Other("Anthropic response had no text content".into()))?;

    let bullets_out = parse_json_array(&text)?;
    if bullets_out.len() != bullets.len() {
        return Err(AppError::Other(format!(
            "AI returned {} bullets but expected {} — keeping original",
            bullets_out.len(),
            bullets.len()
        )));
    }
    Ok(bullets_out)
}

fn build_prompt(bullets: &[String], jd_skills: &[String], jd_excerpt: &str) -> String {
    let numbered: Vec<String> = bullets
        .iter()
        .enumerate()
        .map(|(i, b)| format!("{}. {}", i + 1, b))
        .collect();
    let skills_str = jd_skills.join(", ");
    let jd = jd_excerpt.chars().take(2000).collect::<String>();
    format!(
        "You are tailoring resume bullets to a specific job description.\n\
         \n\
         STRICT RULES:\n\
         1. Preserve every fact: metrics, percentages, dates, company names, technologies, project names, team sizes — exactly as written. Do not change numbers.\n\
         2. Never add accomplishments, skills, technologies, or experiences that aren't in the original bullet.\n\
         3. Only rewrite wording and emphasis to align with the JD's terminology.\n\
         4. Keep each bullet within ~20% of its original length.\n\
         5. Vary the opening verb across bullets; do not start two consecutive bullets with the same word.\n\
         6. If a bullet has nothing relevant to the JD, return it unchanged.\n\
         \n\
         JD KEYWORDS TO EMPHASIZE (when honestly applicable): {skills}\n\
         \n\
         JD EXCERPT:\n\
         {jd}\n\
         \n\
         ORIGINAL BULLETS:\n\
         {numbered}\n\
         \n\
         Respond with ONLY a JSON array of {n} rewritten bullets in the same order. No prose, no markdown, no code fences.",
        skills = skills_str,
        jd = jd,
        numbered = numbered.join("\n"),
        n = bullets.len(),
    )
}

/// Extract the first JSON array from the model's text response.  Tolerates
/// stray prose around it as long as the array itself parses.
fn parse_json_array(text: &str) -> AppResult<Vec<String>> {
    let start = text
        .find('[')
        .ok_or_else(|| AppError::Other("AI response missing JSON array".into()))?;
    let end = text
        .rfind(']')
        .ok_or_else(|| AppError::Other("AI response missing closing ]".into()))?;
    if end < start {
        return Err(AppError::Other("AI response brackets out of order".into()));
    }
    let slice = &text[start..=end];
    let arr: Vec<String> = serde_json::from_str(slice).map_err(|e| {
        AppError::Other(format!("AI JSON parse error: {} — slice: {}", e, truncate(slice, 200)))
    })?;
    Ok(arr)
}

fn truncate(s: &str, n: usize) -> String {
    s.chars().take(n).collect()
}

// ---------------------------------------------------------------------------
// Anthropic API shape
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: u32,
    messages: Vec<Msg>,
}

#[derive(Serialize)]
struct Msg {
    role: &'static str,
    content: String,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    r#type: String,
    text: String,
}
