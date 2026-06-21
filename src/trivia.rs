use anyhow::Result;
use common::source::Source;
use std::time::{SystemTime, UNIX_EPOCH};

mod questions;
mod store;

use questions::QUESTIONS;

/// Maximum number of hints offered per question.
const MAX_HINTS: u32 = 3;

/// Entry point: every trivia sub-command is routed here and dispatched by name.
/// The game is scoped to `s.channel` (the channel the command was used in).
pub fn lookup(s: &Source) -> Result<Vec<String>> {
    match s.command.as_str() {
        "trivia" => toggle(s),
        "question" | "q" => show_question(s),
        "guess" => guess(s),
        "hint" => hint(s),
        "skip" | "next" => skip(s),
        "triviastats" => stats(s),
        _ => Ok(vec![]),
    }
}

// ---------------------------------------------------------------------------
// Sub-commands (DB-backed; exercised against a live MySQL instance in prod).
// ---------------------------------------------------------------------------

fn toggle(s: &Source) -> Result<Vec<String>> {
    if s.query.trim().eq_ignore_ascii_case("off") {
        return Ok(vec![if store::stop_game(&s.channel)? {
            line(
                s,
                &s.c1(format!("Trivia has been turned off by {}.", s.author.nick)),
            )
        } else {
            line(s, &s.c1("Trivia is not enabled here."))
        }]);
    }

    if store::get_game(&s.channel)?.is_some() {
        return Ok(vec![line(s, &s.c1("Trivia is already on!"))]);
    }

    let (question, answer) = QUESTIONS[pick(None)];
    store::start_game(&s.channel, question, answer, now_ms())?;
    Ok(vec![format!(
        "{} {} {} {} {}",
        s.l("Trivia"),
        s.c1(format!("Trivia enabled by {}! Question:", s.author.nick)),
        s.c2(question),
        s.c1("— answer with"),
        s.c2("+guess <answer>")
    )])
}

fn show_question(s: &Source) -> Result<Vec<String>> {
    match store::get_game(&s.channel)? {
        Some(game) => Ok(vec![format!(
            "{} {} {}",
            s.l("Trivia"),
            s.c1("Question:"),
            s.c2(&game.question)
        )]),
        None => Ok(vec![not_enabled(s)]),
    }
}

fn guess(s: &Source) -> Result<Vec<String>> {
    let attempt = s.query.trim();
    if attempt.is_empty() {
        return Ok(vec![line(s, &s.c1("Usage: +guess <answer>"))]);
    }

    let game = match store::get_game(&s.channel)? {
        Some(game) => game,
        None => return Ok(vec![not_enabled(s)]),
    };

    if !answer_matches(attempt, &game.answer) {
        return Ok(vec![format!(
            "{} {} {}",
            s.l("Trivia"),
            s.c2(&s.author.nick),
            s.c1("— not quite, try again!")
        )]);
    }

    let elapsed = (now_ms().saturating_sub(game.asked_at_ms)) as f64 / 1000.0;
    let new_streak = if game.streak_nick == s.author.nick {
        game.streak_len + 1
    } else {
        1
    };
    let outcome = store::record_correct(&s.channel, &s.author.host, &s.author.nick, new_streak)?;

    let (next_q, next_a) = QUESTIONS[pick(current_index(&game.question))];
    store::set_question(&s.channel, next_q, next_a, now_ms())?;

    let mut tail = String::new();
    if outcome.streak_len > 1 {
        tail.push_str(&format!(
            " {}",
            s.c1(format!("Streak: {}", outcome.streak_len))
        ));
    }
    if outcome.new_record {
        tail.push_str(&format!(" {}", s.c2("[NEW RECORD]")));
    }
    if outcome.new_personal_record {
        tail.push_str(&format!(" {}", s.c2("[NEW PB]")));
    }

    Ok(vec![
        format!(
            "{} {} {} {} {} {}{}",
            s.l("Trivia"),
            s.c1(format!("Correct! {} answered in", s.author.nick)),
            s.c2(format!("{:.1}s", elapsed)),
            s.c1("— the answer was"),
            s.c2(&game.answer),
            s.c1(format!(
                "({} correct, {} total)",
                outcome.user_total, outcome.channel_total
            )),
            tail
        ),
        format!("{} {} {}", s.l("Trivia"), s.c1("Next:"), s.c2(next_q)),
    ])
}

fn skip(s: &Source) -> Result<Vec<String>> {
    let game = match store::get_game(&s.channel)? {
        Some(game) => game,
        None => return Ok(vec![not_enabled(s)]),
    };

    let (next_q, next_a) = QUESTIONS[pick(current_index(&game.question))];
    store::set_question(&s.channel, next_q, next_a, now_ms())?;

    Ok(vec![format!(
        "{} {} {} {} {}",
        s.l("Trivia"),
        s.c1(format!("Skipped by {}! The answer was", s.author.nick)),
        s.c2(&game.answer),
        s.c1("— Next:"),
        s.c2(next_q)
    )])
}

fn hint(s: &Source) -> Result<Vec<String>> {
    let game = match store::get_game(&s.channel)? {
        Some(game) => game,
        None => return Ok(vec![not_enabled(s)]),
    };

    if game.hint_level >= MAX_HINTS {
        return Ok(vec![line(
            s,
            &s.c1("I've already given enough hints for this one!"),
        )]);
    }

    let level = store::bump_hint(&s.channel)?;
    Ok(vec![format!(
        "{} {} {}",
        s.l("Trivia"),
        s.c1(format!("Hint {}:", level)),
        s.c2(make_hint(&game.answer, level))
    )])
}

fn stats(s: &Source) -> Result<Vec<String>> {
    let st = store::get_stats(&s.channel, &s.author.host)?;
    let pct = if st.channel_total > 0 {
        st.user_total as f64 / st.channel_total as f64 * 100.0
    } else {
        0.0
    };
    Ok(vec![format!(
        "{} {} {} {} {} {} {} {} {}",
        s.l("Trivia"),
        s.c1("Channel Total:"),
        s.c2(st.channel_total.to_string()),
        s.c1(format!("{} Total:", s.author.nick)),
        s.c2(st.user_total.to_string()),
        s.c1("Best Streak:"),
        s.c2(st.personal_record.to_string()),
        s.c1("Right"),
        s.c2(format!("{:.1}% of the time", pct))
    )])
}

// ---------------------------------------------------------------------------
// Small formatting helpers.
// ---------------------------------------------------------------------------

fn line(s: &Source, body: &str) -> String {
    format!("{} {}", s.l("Trivia"), body)
}

fn not_enabled(s: &Source) -> String {
    line(s, &s.c1(format!("Trivia is not enabled in {}.", s.channel)))
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

/// Finds the table index of the currently-asked question so we can avoid
/// immediately repeating it.
fn current_index(question: &str) -> Option<usize> {
    QUESTIONS.iter().position(|(q, _)| *q == question)
}

/// Time-seeded random question pick, re-rolling once if it matches `exclude`.
fn pick(exclude: Option<usize>) -> usize {
    let seed = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos() as u64)
        .unwrap_or(0);
    pick_index(seed, exclude)
}

// ---------------------------------------------------------------------------
// Pure logic (unit-tested).
// ---------------------------------------------------------------------------

/// SplitMix64 — a tiny, dependency-free PRNG for question selection.
fn splitmix64(seed: u64) -> u64 {
    let mut z = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Maps a seed to a question index, shifting by one if it hits `exclude`.
fn pick_index(seed: u64, exclude: Option<usize>) -> usize {
    let len = QUESTIONS.len();
    let mut idx = (splitmix64(seed) % len as u64) as usize;
    if Some(idx) == exclude {
        idx = (idx + 1) % len;
    }
    idx
}

/// Normalises an answer for comparison: drop dots, trim, lowercase
/// (mirrors mIRC's `$remove(x,.)` + case-insensitive `==`).
fn normalize_answer(s: &str) -> String {
    s.chars()
        .filter(|&c| c != '.')
        .collect::<String>()
        .trim()
        .to_lowercase()
}

fn answer_matches(guess: &str, answer: &str) -> bool {
    normalize_answer(guess) == normalize_answer(answer)
}

/// Reveals a growing prefix of the answer's non-space characters, masking the
/// rest with `-` (spaces preserved). Level 1/2/3 reveal ~25/50/75%.
fn make_hint(answer: &str, level: u32) -> String {
    let non_space = answer.chars().filter(|c| !c.is_whitespace()).count();
    let reveal = ((non_space * level as usize) / 4).clamp(1, non_space.max(1));
    let mut shown = 0;
    let mut out = String::with_capacity(answer.len());
    for c in answer.chars() {
        if c.is_whitespace() {
            out.push(c);
        } else if shown < reveal {
            out.push(c);
            shown += 1;
        } else {
            out.push('-');
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn questions_table_is_populated() {
        assert!(QUESTIONS.len() > 1000);
        assert!(
            QUESTIONS
                .iter()
                .all(|(q, a)| !q.is_empty() && !a.is_empty())
        );
    }

    #[test]
    fn pick_index_is_in_range() {
        for seed in [0u64, 1, 42, 999_999, u64::MAX] {
            assert!(pick_index(seed, None) < QUESTIONS.len());
        }
    }

    #[test]
    fn pick_index_avoids_excluded() {
        for seed in [0u64, 7, 12345, 1 << 40] {
            let natural = pick_index(seed, None);
            let avoided = pick_index(seed, Some(natural));
            assert_ne!(avoided, natural);
            assert!(avoided < QUESTIONS.len());
        }
    }

    #[test]
    fn answers_match_case_dot_and_space_insensitively() {
        assert!(answer_matches("Bronze", "bronze"));
        assert!(answer_matches("5.", "5"));
        assert!(answer_matches("  blue moon  ", "Blue Moon"));
        assert!(!answer_matches("tin", "bronze"));
        assert!(!answer_matches("", "bronze"));
    }

    #[test]
    fn hint_masks_more_as_level_drops_and_preserves_spaces() {
        // "bronze" has 6 letters -> reveal 1 / 3 / 4 at levels 1 / 2 / 3.
        assert_eq!(make_hint("bronze", 1), "b-----");
        assert_eq!(make_hint("bronze", 2), "bro---");
        assert_eq!(make_hint("bronze", 3), "bron--");
        // Spaces are always preserved.
        assert_eq!(
            make_hint("blue moon", 1)
                .chars()
                .filter(|c| *c == ' ')
                .count(),
            1
        );
    }

    #[test]
    fn hint_reveal_is_monotonic() {
        let answer = "Dwarven Stout";
        let revealed = |s: &str| {
            s.chars()
                .filter(|c| *c != '-' && !c.is_whitespace())
                .count()
        };
        let l1 = revealed(&make_hint(answer, 1));
        let l2 = revealed(&make_hint(answer, 2));
        let l3 = revealed(&make_hint(answer, 3));
        assert!(l1 <= l2 && l2 <= l3);
        assert!(l1 >= 1);
    }
}
