//! MySQL persistence for the trivia game.
//!
//! State is keyed by channel because each plugin command opens (and closes) the
//! dylib afresh — no in-process state survives between commands, so everything
//! the game needs to remember lives in the database. Schema (auto-created):
//!
//! ```sql
//! CREATE TABLE IF NOT EXISTS trivia_game (
//!     channel     VARCHAR(64)  NOT NULL PRIMARY KEY,
//!     question    TEXT         NOT NULL,
//!     answer      VARCHAR(255) NOT NULL,
//!     hint_level  INT          NOT NULL DEFAULT 0,
//!     asked_at_ms BIGINT       NOT NULL,
//!     streak_nick VARCHAR(64)  NOT NULL DEFAULT '',
//!     streak_len  INT          NOT NULL DEFAULT 0
//! );
//! CREATE TABLE IF NOT EXISTS trivia_channel (
//!     channel VARCHAR(64) NOT NULL PRIMARY KEY,
//!     total   INT NOT NULL DEFAULT 0,
//!     record  INT NOT NULL DEFAULT 0
//! );
//! CREATE TABLE IF NOT EXISTS trivia_score (
//!     channel     VARCHAR(64)  NOT NULL,
//!     host        VARCHAR(128) NOT NULL,
//!     nick        VARCHAR(64)  NOT NULL DEFAULT '',
//!     correct     INT          NOT NULL DEFAULT 0,
//!     best_streak INT          NOT NULL DEFAULT 0,
//!     PRIMARY KEY (channel, host)
//! );
//! ```

use anyhow::{Context, Result, anyhow};
use mysql::prelude::Queryable;
use mysql::{Conn, params};

use common::database;

/// The active question for a channel.
pub struct Game {
    pub question: String,
    pub answer: String,
    pub hint_level: u32,
    pub asked_at_ms: u64,
    pub streak_nick: String,
    pub streak_len: u32,
}

/// Aggregate outcome of recording a correct answer, used to build the reply.
pub struct Outcome {
    pub user_total: u32,
    pub channel_total: u32,
    pub streak_len: u32,
    pub new_record: bool,
    pub new_personal_record: bool,
}

/// Per-user / per-channel stats for the `triviastats` command.
pub struct Stats {
    pub channel_total: u32,
    pub user_total: u32,
    pub personal_record: u32,
}

fn conn() -> Result<Conn> {
    let mut c = database::connect().map_err(|e| anyhow!("database connection failed: {}", e))?;
    ensure_schema(&mut c)?;
    Ok(c)
}

fn ensure_schema(c: &mut Conn) -> Result<()> {
    c.query_drop(
        "CREATE TABLE IF NOT EXISTS trivia_game (
            channel     VARCHAR(64)  NOT NULL PRIMARY KEY,
            question    TEXT         NOT NULL,
            answer      VARCHAR(255) NOT NULL,
            hint_level  INT          NOT NULL DEFAULT 0,
            asked_at_ms BIGINT       NOT NULL,
            streak_nick VARCHAR(64)  NOT NULL DEFAULT '',
            streak_len  INT          NOT NULL DEFAULT 0
        )",
    )
    .context("create trivia_game")?;
    c.query_drop(
        "CREATE TABLE IF NOT EXISTS trivia_channel (
            channel VARCHAR(64) NOT NULL PRIMARY KEY,
            total   INT NOT NULL DEFAULT 0,
            record  INT NOT NULL DEFAULT 0
        )",
    )
    .context("create trivia_channel")?;
    c.query_drop(
        "CREATE TABLE IF NOT EXISTS trivia_score (
            channel     VARCHAR(64)  NOT NULL,
            host        VARCHAR(128) NOT NULL,
            nick        VARCHAR(64)  NOT NULL DEFAULT '',
            correct     INT          NOT NULL DEFAULT 0,
            best_streak INT          NOT NULL DEFAULT 0,
            PRIMARY KEY (channel, host)
        )",
    )
    .context("create trivia_score")?;
    Ok(())
}

/// Returns the active game for a channel, or `None` if trivia is off there.
pub fn get_game(channel: &str) -> Result<Option<Game>> {
    let mut c = conn()?;
    let row: Option<(String, String, u32, u64, String, u32)> = c
        .exec_first(
            "SELECT question, answer, hint_level, asked_at_ms, streak_nick, streak_len
             FROM trivia_game WHERE channel = :channel",
            params! { "channel" => channel },
        )
        .context("select trivia_game")?;

    Ok(row.map(
        |(question, answer, hint_level, asked_at_ms, streak_nick, streak_len)| Game {
            question,
            answer,
            hint_level,
            asked_at_ms,
            streak_nick,
            streak_len,
        },
    ))
}

/// Starts (or restarts) a game with the first question. Resets hint and streak.
pub fn start_game(channel: &str, question: &str, answer: &str, now_ms: u64) -> Result<()> {
    let mut c = conn()?;
    c.exec_drop(
        "REPLACE INTO trivia_game
            (channel, question, answer, hint_level, asked_at_ms, streak_nick, streak_len)
         VALUES (:channel, :question, :answer, 0, :now, '', 0)",
        params! {
            "channel" => channel,
            "question" => question,
            "answer" => answer,
            "now" => now_ms,
        },
    )
    .context("start trivia_game")
}

/// Advances to a new question, keeping the running streak intact.
pub fn set_question(channel: &str, question: &str, answer: &str, now_ms: u64) -> Result<()> {
    let mut c = conn()?;
    c.exec_drop(
        "UPDATE trivia_game
            SET question = :question, answer = :answer, hint_level = 0, asked_at_ms = :now
          WHERE channel = :channel",
        params! {
            "channel" => channel,
            "question" => question,
            "answer" => answer,
            "now" => now_ms,
        },
    )
    .context("advance trivia_game")
}

/// Stops the game for a channel. Returns true if one was running.
pub fn stop_game(channel: &str) -> Result<bool> {
    let mut c = conn()?;
    c.exec_drop(
        "DELETE FROM trivia_game WHERE channel = :channel",
        params! { "channel" => channel },
    )
    .context("stop trivia_game")?;
    Ok(c.affected_rows() > 0)
}

/// Increments the hint level and returns the new value.
pub fn bump_hint(channel: &str) -> Result<u32> {
    let mut c = conn()?;
    c.exec_drop(
        "UPDATE trivia_game SET hint_level = hint_level + 1 WHERE channel = :channel",
        params! { "channel" => channel },
    )
    .context("bump hint")?;
    let level: Option<u32> = c
        .exec_first(
            "SELECT hint_level FROM trivia_game WHERE channel = :channel",
            params! { "channel" => channel },
        )
        .context("select hint level")?;
    Ok(level.unwrap_or(0))
}

/// Records a correct answer: updates the player's tally, the channel total, the
/// streak, and channel/personal records. `new_streak` is computed by the caller.
pub fn record_correct(channel: &str, host: &str, nick: &str, new_streak: u32) -> Result<Outcome> {
    let mut c = conn()?;

    // Player tally + personal-best streak.
    let prior: Option<(u32, u32)> = c
        .exec_first(
            "SELECT correct, best_streak FROM trivia_score WHERE channel = :channel AND host = :host",
            params! { "channel" => channel, "host" => host },
        )
        .context("select trivia_score")?;
    let (prior_correct, prior_best) = prior.unwrap_or((0, 0));
    let user_total = prior_correct + 1;
    let best_streak = prior_best.max(new_streak);
    let new_personal_record = new_streak > prior_best;
    c.exec_drop(
        "INSERT INTO trivia_score (channel, host, nick, correct, best_streak)
         VALUES (:channel, :host, :nick, :correct, :best)
         ON DUPLICATE KEY UPDATE correct = :correct, nick = :nick, best_streak = :best",
        params! {
            "channel" => channel,
            "host" => host,
            "nick" => nick,
            "correct" => user_total,
            "best" => best_streak,
        },
    )
    .context("upsert trivia_score")?;

    // Channel total + record streak.
    let prior_chan: Option<(u32, u32)> = c
        .exec_first(
            "SELECT total, record FROM trivia_channel WHERE channel = :channel",
            params! { "channel" => channel },
        )
        .context("select trivia_channel")?;
    let (prior_total, prior_record) = prior_chan.unwrap_or((0, 0));
    let channel_total = prior_total + 1;
    let record = prior_record.max(new_streak);
    let new_record = new_streak > prior_record;
    c.exec_drop(
        "INSERT INTO trivia_channel (channel, total, record)
         VALUES (:channel, :total, :record)
         ON DUPLICATE KEY UPDATE total = :total, record = :record",
        params! { "channel" => channel, "total" => channel_total, "record" => record },
    )
    .context("upsert trivia_channel")?;

    // Persist the running streak on the game row.
    c.exec_drop(
        "UPDATE trivia_game SET streak_nick = :nick, streak_len = :len WHERE channel = :channel",
        params! { "channel" => channel, "nick" => nick, "len" => new_streak },
    )
    .context("update streak")?;

    Ok(Outcome {
        user_total,
        channel_total,
        streak_len: new_streak,
        new_record,
        new_personal_record,
    })
}

/// Stats for a given player host within a channel.
pub fn get_stats(channel: &str, host: &str) -> Result<Stats> {
    let mut c = conn()?;
    let channel_total: Option<u32> = c
        .exec_first(
            "SELECT total FROM trivia_channel WHERE channel = :channel",
            params! { "channel" => channel },
        )
        .context("select channel total")?;
    let user: Option<(u32, u32)> = c
        .exec_first(
            "SELECT correct, best_streak FROM trivia_score WHERE channel = :channel AND host = :host",
            params! { "channel" => channel, "host" => host },
        )
        .context("select user score")?;
    let (user_total, personal_record) = user.unwrap_or((0, 0));
    Ok(Stats {
        channel_total: channel_total.unwrap_or(0),
        user_total,
        personal_record,
    })
}
