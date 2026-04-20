use std::{env, process::Command};

use anyhow::Result;
use serde::{Deserialize, Serialize};

const START_TIME: &str = "START_TIME";
const END_TIME: &str = "END_TIME";
const START_TIME_WEIGHT: f32 = 0.7;
const COMMANDS_USED: &str = "NUMBER_OF_USED_COMMANDS";
const COMMANDS_USED_WEIGHT: f32 = 0.3;
const GOLD_SCORE_LIMIT: f32 = 7.;
const SILVER_SCORE_LIMIT: f32 = 20.;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Rank {
    Gold,
    Silver,
    Bronze,
}

impl Rank {
    pub fn as_str(&self) -> &'static str {
        match self {
            Rank::Gold => "Gold",

            Rank::Silver => "Silver",

            Rank::Bronze => "Bronze",
        }
    }
}

// to 2 decimal points
pub fn calculate_time() -> Result<f32> {
    let start_time = env::var(START_TIME)?.parse::<f32>()?;
    let end_time = env::var(END_TIME)?.parse::<f32>()?;

    let time = ((end_time - start_time) * 100.).round() / 100.;

    Ok(time)
}

pub fn fetch_command_count() -> Result<i32> {
    let count = env::var(COMMANDS_USED)?.parse::<i32>()?;

    Ok(count)
}

pub fn calculate_score(time: f32, command_count: i32) -> Result<i32> {
    // admittedly arbritrary calculation to make lower times and commands used give higher scores
    // could be point of contention
    // other params like weights can be fine tuned in the future
    let score = ((1.
        / ((time * START_TIME_WEIGHT) * (command_count as f32 * COMMANDS_USED_WEIGHT)))
        * 100.)
        .round() as i32;

    Ok(score)
}

pub fn assign_rank(score: i32) -> Result<Rank> {
    let calculated_score = 1. / (score as f32 / 100.);
    if calculated_score <= GOLD_SCORE_LIMIT {
        Ok(Rank::Gold)
    } else if calculated_score <= SILVER_SCORE_LIMIT {
        Ok(Rank::Silver)
    } else {
        Ok(Rank::Bronze)
    }
}
