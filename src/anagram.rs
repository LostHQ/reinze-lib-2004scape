use anyhow::Result;
use common::source::Source;

pub fn lookup(s: &Source) -> Result<Vec<String>> {
    if s.query.trim().is_empty() {
        return Ok(vec![format!(
            "{} {}",
            s.l("Anagram"),
            s.c1("You must specify an anagram.")
        )]);
    }

    // Key = the query with all spaces removed, matched case-insensitively.
    let key = s.query.replace(' ', "");

    let output = match find_solution(&key) {
        Some((person, location)) => format!(
            "{} {} {} {} {}",
            s.l("Anagram"),
            s.c1("Person:"),
            s.c2(person),
            s.c1("Location:"),
            s.c2(location)
        ),
        None => format!(
            "{} {} {}",
            s.l("Anagram"),
            s.c1("No matches for"),
            s.c2(format!("\"{}\"", s.query))
        ),
    };

    Ok(vec![output])
}

/// Resolves a space-stripped anagram key to its (person, location) solution.
fn find_solution(key: &str) -> Option<(&'static str, &'static str)> {
    Anagram::ALL.iter().find_map(|anagram| {
        let (anagram_key, person, location) = anagram.details();
        anagram_key
            .eq_ignore_ascii_case(key)
            .then_some((person, location))
    })
}

/// A treasure-trail anagram clue, ported from the mIRC `[Anagrams]` table.
/// Each variant is named after the NPC the clue resolves to.
enum Anagram {
    Saba,
    ZeneshaNew,
    CamTheCamel,
    Jaraah,
    CaptainNintoNew,
    Caroline,
    Oracle,
    RamaraDuCroissantNew,
    Brimstail,
    Bolkoy,
    GnomeCoach,
    Brundt,
    Zookeeper,
    Lowe,
    Recruiter,
    KingBolren,
    Gabooty,
    UglugNarNew,
    Luthas,
    RikiTheSculptorSModelNew,
    Fycie,
    ShirattiTheCustodianNew,
    FairyNuffNew,
    OddOldManNew,
    KingRoald,
    CamTheCamelNew,
    Femi,
    Edmond,
    CapNIzzyNoBeard,
    Cook,
    WizardFrumsconeNew,
    ProfessorOnglewipNew,
    PartyPete,
    Karim,
    TraderStanNew,
    QueenSigridNew,
    Hans,
    CamTheCamel2,
}

impl Anagram {
    const ALL: &'static [Anagram] = &[
        Anagram::Saba,
        Anagram::ZeneshaNew,
        Anagram::CamTheCamel,
        Anagram::Jaraah,
        Anagram::CaptainNintoNew,
        Anagram::Caroline,
        Anagram::Oracle,
        Anagram::RamaraDuCroissantNew,
        Anagram::Brimstail,
        Anagram::Bolkoy,
        Anagram::GnomeCoach,
        Anagram::Brundt,
        Anagram::Zookeeper,
        Anagram::Lowe,
        Anagram::Recruiter,
        Anagram::KingBolren,
        Anagram::Gabooty,
        Anagram::UglugNarNew,
        Anagram::Luthas,
        Anagram::RikiTheSculptorSModelNew,
        Anagram::Fycie,
        Anagram::ShirattiTheCustodianNew,
        Anagram::FairyNuffNew,
        Anagram::OddOldManNew,
        Anagram::KingRoald,
        Anagram::CamTheCamelNew,
        Anagram::Femi,
        Anagram::Edmond,
        Anagram::CapNIzzyNoBeard,
        Anagram::Cook,
        Anagram::WizardFrumsconeNew,
        Anagram::ProfessorOnglewipNew,
        Anagram::PartyPete,
        Anagram::Karim,
        Anagram::TraderStanNew,
        Anagram::QueenSigridNew,
        Anagram::Hans,
        Anagram::CamTheCamel2,
    ];

    /// Returns the space-stripped anagram key, the person, and the location.
    fn details(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Anagram::Saba => (
                "ABAS",
                "Saba",
                "Northwest of Burthorpe (north of the Hero's Guild) in an unmarked cave to the Northwest of the Slayer Master",
            ),
            Anagram::ZeneshaNew => (
                "AZENSHE",
                "Zenesha NEW",
                "East Ardougne market - platebody seller",
            ),
            Anagram::CamTheCamel => (
                "ACEMATCHELM",
                "Cam the Camel",
                "Al Kharid outside Duel Arena gate, Camulet *NOT* required.",
            ),
            Anagram::Jaraah => ("AHAJAR", "Jaraah", "Dueling Arena hospital"),
            Anagram::CaptainNintoNew => (
                "ANPAINTTONIC",
                "Captain Ninto NEW",
                "Dwarf Passage under White Wolf Mountain, near the beer area. (Requires Fishing Contest to enter.)",
            ),
            Anagram::Caroline => (
                "ARCOLINE",
                "Caroline",
                "East of Ardougne on the coast, at the Sea Slug quest start. After Kennith's Concerns quest she is upstairs in Kennith's house.",
            ),
            Anagram::Oracle => ("ARECOL", "Oracle", "Ice Mountain North of Falador"),
            Anagram::RamaraDuCroissantNew => (
                "ARR!SOIAMACRUST,AND?",
                "Ramara du Croissant NEW",
                "Piscatoris forge (requires finishing Swan Song quest)",
            ),
            Anagram::Brimstail => (
                "BAILTRIMS",
                "Brimstail",
                "Gnome Stronghold, East of the bridge on the West side of the Stronghold. Enter the hollowed rock, which leads to his cave",
            ),
            Anagram::Bolkoy => (
                "BYLOOK",
                "Bolkoy",
                "Upstairs in the Tree Gnome Village (the Maze)",
            ),
            Anagram::GnomeCoach => (
                "CONGAMEHOC",
                "Gnome Coach",
                "Gnome Stronghold, outside the Gnomeball Field (he circles the whole field)",
            ),
            Anagram::Brundt => (
                "DTRUNB",
                "Brundt",
                "Rellekka chieftain, inside the Longhall in Rellekka",
            ),
            Anagram::Zookeeper => ("EEKZEROOP", "Zookeeper", "Ardougne"),
            Anagram::Lowe => ("ELOW", "Lowe", "Archery Store in Varrock"),
            Anagram::Recruiter => (
                "ERRCUREIT",
                "Recruiter",
                "Centre of town, West Ardougne. You must have at least started the Plague City Quest to get into West Ardougne.",
            ),
            Anagram::KingBolren => (
                "GOBLINKERN",
                "King Bolren",
                "Centre of Tree Gnome Village (the Maze)",
            ),
            Anagram::Gabooty => ("GOTABOY", "Gabooty", "Tai Bwo Wannai Village"),
            Anagram::UglugNarNew => (
                "GULAGRUN",
                "Uglug Nar NEW",
                "South of Castle Wars at Jiggig.",
            ),
            Anagram::Luthas => ("HALTUS", "Luthas", "Karamja Banana Plantation"),
            Anagram::RikiTheSculptorSModelNew => (
                "HEDOPOSE.ITISCULTRRL,MK?",
                "Riki the sculptor's model NEW",
                "Keldagrim Sculptor's studio, on northeast side of town.",
            ),
            Anagram::Fycie => (
                "ICYFE",
                "Fycie",
                "In a cave in Ogre Country South of Gu'tanoth, directly to the North of NPC Rantz, in the valley where the Chompy Bird Hunting quest start is",
            ),
            Anagram::ShirattiTheCustodianNew => (
                "IEATITSCHARTHINTSDOU",
                "Shiratti the Custodian NEW",
                "Nardah",
            ),
            Anagram::FairyNuffNew => (
                "IFAFFYRUN",
                "Fairy Nuff NEW",
                "North of Lost City (Zanaris) bank. (However, if you've started the Fairy Tale part 2 quest, get the certificate from the room north of the bank, then use Fairy Ring codes AIR, DLR, DJQ, AJS.) (Clue requires Lost City quest.)",
            ),
            Anagram::OddOldManNew => (
                "LANDDOOMD",
                "Odd Old Man NEW",
                "Rag and Bone Man quest start, road east of Varrock Earth Runecrafting Altar, north of Limestone Mine.",
            ),
            Anagram::KingRoald => ("LARKINDOG", "King Roald ", "Varrock Palace"),
            Anagram::CamTheCamelNew => (
                "MEAMTHECALC",
                "Cam the Camel NEW",
                "Al Kharid outside Duel Arena gate, Camulet *NOT* required.",
            ),
            Anagram::Femi => (
                "MEIF",
                "Femi",
                "Next to the cart at the gate to Tree Gnome Stronghold",
            ),
            Anagram::Edmond => (
                "NODMED",
                "Edmond",
                "East Ardougne, in the house North of the Castle",
            ),
            Anagram::CapNIzzyNoBeard => (
                "OBIRDZAZANYENPC",
                "Cap'n Izzy No Beard",
                "Brimhaven Agility Arena",
            ),
            Anagram::Cook => ("OKCO", "Cook", "Lumbridge Castle"),
            Anagram::WizardFrumsconeNew => (
                "ORZINCFUMESWARD",
                "Wizard Frumscone NEW",
                "Yanille Magic Guild basement, need 66 magic (boosters allowed) to enter.",
            ),
            Anagram::ProfessorOnglewipNew => (
                "PROFSLOSEWRONGPIE",
                "Professor Onglewip NEW",
                "Draynor wizard tower ground floor or outside",
            ),
            Anagram::PartyPete => ("PEATYPERT", "Party Pete", "Falador"),
            Anagram::Karim => ("RAKMI", "Karim", "Al-Kharid kebab seller"),
            Anagram::TraderStanNew => (
                "REDARTTANS",
                "Trader Stan NEW",
                "Owner of Charter Ships, speak to him at the southern dock in Port Sarim.",
            ),
            Anagram::QueenSigridNew => (
                "SEQUINDIRGE",
                "Queen Sigrid NEW",
                "Queen of Etceteria, must have completed Fremennik Trials quest to go there.",
            ),
            Anagram::Hans => ("SNAH", "Hans", "The confused lad around Lumbridge Castle"),
            Anagram::CamTheCamel2 => (
                "THEMCALLCAME",
                "Cam the Camel",
                "Al Kharid outside Duel Arena gate, Camulet *NOT* required.",
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn solves_a_known_anagram() {
        let s = src("a bas");
        let output = lookup(&s).unwrap();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("Person:"));
        assert!(output[0].contains("Saba"));
        assert!(output[0].contains("Location:"));
        assert!(output[0].contains("Burthorpe"));
    }

    #[test]
    fn solves_anagram_ignoring_case_and_spaces() {
        let upper = lookup(&src("ARECOL")).unwrap();
        let spaced = lookup(&src("ar ecol")).unwrap();
        assert!(upper[0].contains("Oracle"));
        assert!(upper[0].contains("Ice Mountain North of Falador"));
        assert_eq!(upper, spaced);
    }

    #[test]
    fn reports_no_match_with_quoted_query() {
        let s = src("not an anagram");
        let output = lookup(&s).unwrap();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("No matches for"));
        assert!(output[0].contains("\"not an anagram\""));
    }

    #[test]
    fn requires_an_anagram_when_empty() {
        let s = src("");
        let output = lookup(&s).unwrap();
        assert_eq!(output.len(), 1);
        assert!(output[0].contains("You must specify an anagram."));
    }

    #[test]
    fn every_anagram_is_well_formed_and_unique() {
        let mut seen = std::collections::HashSet::new();
        for anagram in Anagram::ALL {
            let (key, person, location) = anagram.details();
            assert!(!key.is_empty(), "empty key");
            assert!(!person.is_empty(), "empty person for {key}");
            assert!(!location.is_empty(), "empty location for {key}");
            assert!(!key.contains(' '), "key has spaces: {key}");
            assert!(seen.insert(key.to_uppercase()), "duplicate key: {key}");
        }
        assert_eq!(Anagram::ALL.len(), 38);
    }

    // --- test helpers ---

    use common::ColorResult;
    use common::author::Author;
    use std::os::raw::c_char;

    extern "C" fn stub_color(_host: *const c_char, _colors: *const c_char) -> ColorResult {
        ColorResult::default()
    }

    fn src(query: &str) -> Source {
        Source::create(
            "0",
            Author::create("nick!ident@host", stub_color),
            "anagram",
            query,
        )
    }
}
