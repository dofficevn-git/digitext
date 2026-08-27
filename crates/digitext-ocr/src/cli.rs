// Copyright (c) 2026 DigiText. All rights reserved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Run,
    Health,
    Version,
    Help,
    Unknown(String),
}
pub fn parse<I, S>(r_30l: I) -> Command
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut r_30m = r_30l.into_iter();
    let _bin = r_30m.next();
    match r_30m.next() {
        None => Command::Run,
        Some(r_30k) => match r_30k.as_ref() {
            "run" => Command::Run,
            "--health" | "health" => Command::Health,
            "--version" | "-V" => Command::Version,
            "--help" | "-h" => Command::Help,
            r_30n => Command::Unknown(r_30n.to_owned()),
        },
    }
}
