extern crate core;

mod anagram;
mod boost;
mod challenge;
mod common;
mod coords;
mod level;
mod noburn;
mod players;
mod rsn;
mod speakto;
mod spell;
mod stats;
mod track;
mod trivia;
mod xp;

use ::common::author::Author;
use ::common::source::Source;
use ::common::{PluginContext, to_str_or_default};
use log::error;
use regex::Regex;
use std::ffi::CString;
use std::os::raw::c_char;

/// Regex triggers this plugin claims, one per line. The host invokes the plugin
/// once per matching trigger, so these must not overlap for a single command
/// (e.g. anything containing "stats" is already covered by the `stats` line).
const TRIGGERS: &str = r"boosts?
anagram
challenge
(coords?|clue)
co?mb(at)?\d*$
x?e?xp(erience)?
le?ve?l
(no)?burn
spell
speak(to)?
^players?$
rsn\d*
stats
overall
total
att(ack)?
def(ence)?
str(ength)?
h(it)?p(oints)?
ranged?
pray(er)?
mag(e|ic)
cook(ing)?
w(ood)?c(utting)?
fletch(ing)?
fish(ing)?
f(ire)?m(aking)?
craft(ing)?
smith(ing)?
min(e|ing)
herb(lore)?
agil(ity)?
thie(f|ving)
r(une)?c(raft)?
track\d*
^trivia$
^(question|q)$
^guess$
^hint$
^(skip|next)$";

#[unsafe(no_mangle)]
pub extern "C" fn exported(context: *const PluginContext) -> *mut c_char {
    unsafe {
        let nil = CString::new("").unwrap().into_raw();

        if context.is_null() {
            return nil;
        }

        let mut command = to_str_or_default((*context).cmd);
        let query = to_str_or_default((*context).param);
        let author = to_str_or_default((*context).author);
        let color = (*context).color;
        let channel = to_str_or_default((*context).channel);

        let re = Regex::new(r"^([a-zA-Z]+)(\d+)$").unwrap();
        let cmd = command.to_string();
        let re_match = match re.captures(&cmd) {
            Some(captures) => vec![captures],
            None => vec![],
        };

        let mut rsn_n = "0";

        if re_match.len() > 0 {
            command = re_match[0].get(1).unwrap().as_str().to_string();
            rsn_n = re_match[0].get(2).unwrap().as_str();
        }

        let source = Source::create(
            rsn_n,
            Author::create(author, color),
            &command.to_string(),
            &query,
        )
        .with_channel(channel);

        match match command.as_str() {
            "overall" | "stats" | "total" | "attack" | "att" | "defence" | "def" | "strength"
            | "str" | "hitpoints" | "hp" | "ranged" | "range" | "prayer" | "pray" | "magic"
            | "mage" | "cooking" | "cook" | "woodcutting" | "wc" | "fletching" | "fletch"
            | "fishing" | "fish" | "firemaking" | "fm" | "crafting" | "craft" | "smithing"
            | "smith" | "mining" | "mine" | "herblore" | "herb" | "agility" | "agil"
            | "thieving" | "thief" | "runecraft" | "rc" => stats::lookup(source),
            "boost" | "boosts" => boost::lookup(&source),
            "anagram" => anagram::lookup(&source),
            "challenge" => challenge::lookup(&source),
            "coords" | "coord" | "clue" => coords::lookup(&source),
            "combat" | "cmb" => stats::combat(source),
            "experience" | "xperience" | "exp" | "xp" => xp::lookup(&source),
            "level" | "lvl" => level::lookup(&source),
            "noburn" | "burn" => noburn::noburn(&source),
            "spell" => spell::lookup(&source),
            "speakto" | "speak" => speakto::lookup(&source),
            "players" => players::lookup(&source),
            "rsn" => rsn::process(source),
            "track" => track::lookup(source),
            "tracksnapshot" => track::snapshot_all(),
            // NOTE: "triviastats" intentionally has no dedicated trigger — the
            // unanchored `stats` trigger already routes it here. Adding a second
            // matching trigger makes the host invoke the plugin twice (duplicate
            // output), since it calls once per matching trigger.
            "trivia" | "question" | "q" | "guess" | "hint" | "skip" | "next" | "triviastats" => {
                trivia::lookup(&source)
            }
            "help" => Ok(r"boost
anagram
challenge
coords
combat[N]
exp
level
noburn
spell
speakto
players
rsn[N]
stats[N]
track[N]
trivia [on|off]
question
guess
hint
skip
triviastats"
                .split("\n")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()),
            "" => Ok(TRIGGERS
                .split("\n")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()),
            "timers" => Ok(vec!["tracksnapshot:6h".to_string()]),
            _ => Ok(vec![]),
        } {
            Ok(output) => match CString::new(output.join("\n")) {
                Ok(output) => output.into_raw(),
                Err(_) => nil,
            },
            Err(e) => {
                error!("Command '{}' failed: {:?}", command, e);
                nil
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use regex::Regex;

    /// Mirrors the host: a trigger matches a command if its regex matches anywhere.
    fn matching_triggers(cmd: &str) -> usize {
        super::TRIGGERS
            .split('\n')
            .filter(|line| Regex::new(line).map(|re| re.is_match(cmd)).unwrap_or(false))
            .count()
    }

    #[test]
    fn every_trivia_command_matches_exactly_one_trigger() {
        // The host calls the plugin once per matching trigger; more than one match
        // duplicates output (the bug that hit `triviastats` via the `stats` line).
        for cmd in [
            "trivia",
            "question",
            "q",
            "guess",
            "hint",
            "skip",
            "next",
            "triviastats",
        ] {
            assert_eq!(
                matching_triggers(cmd),
                1,
                "`{cmd}` should match exactly one trigger"
            );
        }
    }

    #[test]
    fn stats_commands_remain_single_match() {
        assert_eq!(matching_triggers("stats"), 1);
        assert_eq!(matching_triggers("stats2"), 1);
    }
}
