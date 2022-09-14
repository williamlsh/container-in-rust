use anyhow::{anyhow, Result};
use nix::unistd::sethostname;
use rand::{seq::SliceRandom, Rng};

const HOSTNAME_NAMES: [&str; 8] = [
    "cat", "world", "coffee", "girl", "man", "book", "penguin", "moon",
];

const HOSTNAME_ADJ: [&str; 16] = [
    "blue",
    "red",
    "green",
    "yellow",
    "big",
    "small",
    "tall",
    "thin",
    "round",
    "square",
    "triangular",
    "weird",
    "noisy",
    "silent",
    "soft",
    "irregular",
];

pub(crate) fn generate_hostname() -> Result<String> {
    let mut rng = rand::thread_rng();
    let num: u8 = rng.gen();
    let name = HOSTNAME_NAMES
        .choose(&mut rng)
        .ok_or_else(|| anyhow!("Could not choose hostname"))?;
    let adj = HOSTNAME_ADJ
        .choose(&mut rng)
        .ok_or_else(|| anyhow!("Could not choose address"))?;
    Ok(format!("{}-{}-{}", adj, name, num))
}

pub(crate) fn set_container_hostname(hostname: &String) -> nix::Result<()> {
    sethostname(hostname)
}
