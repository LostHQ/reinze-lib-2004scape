extern crate core;

mod anagram;
mod boost;
mod common;
mod coords;
mod level;
mod noburn;
mod rsn;
mod spell;
mod stats;
mod track;
mod xp;

use ::common::author::Author;
use ::common::source::Source;
use ::common::{PluginContext, to_str_or_default};
use log::error;
use regex::Regex;
use std::ffi::CString;
use std::os::raw::c_char;

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
        );

        match match command.as_str() {
            "overall" | "stats" | "total" | "attack" | "att" | "defence" | "def" | "strength"
            | "str" | "hitpoints" | "hp" | "ranged" | "range" | "prayer" | "pray" | "magic"
            | "mage" | "cooking" | "cook" | "woodcutting" | "wc" | "fletching" | "fletch"
            | "fishing" | "fish" | "firemaking" | "fm" | "crafting" | "craft" | "smithing"
            | "smith" | "mining" | "mine" | "herblore" | "herb" | "agility" | "agil"
            | "thieving" | "thief" | "runecraft" | "rc" => stats::lookup(source),
            "boost" | "boosts" => boost::lookup(&source),
            "anagram" => anagram::lookup(&source),
            "coords" | "coord" | "clue" => coords::lookup(&source),
            "combat" | "cmb" => stats::combat(source),
            "experience" | "xperience" | "exp" | "xp" => xp::lookup(&source),
            "level" | "lvl" => level::lookup(&source),
            "noburn" | "burn" => noburn::noburn(&source),
            "spell" => spell::lookup(&source),
            "rsn" => rsn::process(source),
            "track" => track::lookup(source),
            "tracksnapshot" => track::snapshot_all(),
            "help" => Ok(r"boost
anagram
coords
combat[N]
exp
level
noburn
spell
rsn[N]
stats[N]
track[N]"
                .split("\n")
                .map(|s| s.to_string())
                .collect::<Vec<String>>()),
            "" => Ok(r"boosts?
anagram
(coords?|clue)
co?mb(at)?\d*$
x?e?xp(erience)?
le?ve?l
(no)?burn
spell
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
track\d*"
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
