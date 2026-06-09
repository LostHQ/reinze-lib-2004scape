use anyhow::Result;
use common::source::Source;

pub fn noburn(s: &Source) -> Result<Vec<String>> {
    let query = &s.query;

    let fish: Vec<Fish> = vec![
        Fish::new("Tuna", 63, 0, 0),
        Fish::new("Lobster", 74, 74, 64),
        Fish::new("Swordfish", 86, 81, 81),
        Fish::new("Shark", 0, 0, 94),
    ];

    let output: Vec<String> = fish
        .into_iter()
        .filter(|fish| fish_finder(fish, query))
        .map(|fish| fish.to_string(s))
        .collect();

    Ok(vec![
        format!("{} {}", s.l("NoBurn"), output.join(&s.c1(" | "))),
        s.p("Fire | Range | Gauntlets"),
    ])
}

struct Fish {
    name: &'static str,
    fire: u32,
    range: u32,
    gauntlets: u32,
}

impl Fish {
    fn new(name: &'static str, fire: u32, range: u32, gauntlets: u32) -> Self {
        Self {
            name,
            fire,
            range,
            gauntlets,
        }
    }

    fn to_string(&self, s: &Source) -> String {
        vec![
            s.c1(self.name),
            s.c2(&if_not_available(self.fire)),
            s.c2(&if_not_available(self.range)),
            s.c2(&if_not_available(self.gauntlets)),
        ]
        .join(" ")
    }
}

fn if_not_available(int: u32) -> String {
    if int == 0 {
        "N/A".to_string()
    } else {
        int.to_string()
    }
}

fn fish_finder(fish: &Fish, query: &str) -> bool {
    (query.len() > 0 && fish.name.to_lowercase().contains(&query.to_lowercase()))
        || query.len() == 0
}
