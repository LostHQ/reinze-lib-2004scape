use anyhow::{Context, Result, bail};
use common::source::Source;
use log::error;
use regex::Regex;
use reqwest::header::USER_AGENT;
use std::time::Duration;

const TITLE_URL: &str = "https://2004.lostcity.rs/title";

pub fn lookup(s: &Source) -> Result<Vec<String>> {
    let body = match fetch_title() {
        Ok(body) => body,
        Err(e) => {
            error!("players fetch failed: {}", e);
            return Ok(vec![format!(
                "{} {}",
                s.l("Players"),
                s.c1("Could not reach the server right now.")
            )]);
        }
    };

    match parse_player_count(&body) {
        Some(count) => Ok(vec![format!(
            "{} {} {} {}",
            s.l("Players"),
            s.c1("There are currently"),
            s.c2(count),
            s.c1("people playing!")
        )]),
        None => Ok(vec![format!(
            "{} {}",
            s.l("Players"),
            s.c1("Could not determine the current player count.")
        )]),
    }
}

fn fetch_title() -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .connect_timeout(Duration::new(5, 0))
        .build()
        .context("failed to build HTTP client")?;

    let resp = match client
        .get(TITLE_URL)
        .header(USER_AGENT, "Reinze.com")
        .send()
    {
        Ok(resp) => resp,
        Err(e) => {
            error!("{}", e);
            bail!("failed to make players HTTP request");
        }
    };

    resp.text().context("failed to read players response body")
}

/// Extracts the player count from the title page's
/// "There are currently N people playing!" line.
fn parse_player_count(body: &str) -> Option<u32> {
    let re = Regex::new(r"(?i)currently\s+([\d,]+)\s+people\s+playing").ok()?;
    let digits = re.captures(body)?.get(1)?.as_str().replace(',', "");
    digits.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_count_from_title_span() {
        let html = r#"<span style="font-size:14px; color:white;">There are currently 218 people playing!</span>"#;
        assert_eq!(parse_player_count(html), Some(218));
    }

    #[test]
    fn parses_count_with_thousands_separator() {
        assert_eq!(
            parse_player_count("There are currently 1,234 people playing!"),
            Some(1234)
        );
    }

    #[test]
    fn returns_none_when_absent() {
        assert_eq!(parse_player_count("<html>no count here</html>"), None);
    }
}
