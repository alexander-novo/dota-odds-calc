use std::{error::Error, path::PathBuf};

use clap::{Parser, ValueEnum};
use csv::Writer;

#[derive(ValueEnum, Clone, Debug)]
enum Rarity {
    Rare,
    VeryRare,
    UltraRare,
}

const MAX_ODDS: usize = 50;

impl Rarity {
    fn odds(&self) -> &[f32; MAX_ODDS] {
        match self {
            Rarity::Rare => &[
                20_000., 583., 187., 88., 51., 33., 23., 17., 13.1, 10.4, 8.5, 7.1, 6.0, 5.2, 4.5,
                4.0, 3.6, 3.2, 2.9, 2.6, 2.4, 2.2, 2.1, 1.9, 1.8, 1.7, 1.6, 1.5, 1.5, 1.4, 1.3,
                1.3, 1.2, 1.2, 1.2, 1.1, 1.1, 1.1, 1.1, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0,
                1.0, 1.0, 1.0,
            ],
            Rarity::VeryRare => &[
                20_000., 3_653., 1_059., 485., 276., 178., 124., 92., 70., 56., 45., 38., 32., 27.,
                24., 21., 18., 16., 14.1, 12.7, 11.5, 10.5, 9.6, 8.8, 8.1, 7.5, 7.0, 6.5, 6.0, 5.7,
                5.3, 5.0, 4.7, 4.5, 4.2, 4.0, 3.8, 3.6, 3.4, 3.3, 3.2, 3.0, 2.9, 2.8, 2.7, 2.6,
                2.5, 2.4, 2.3, 2.2,
            ],
            Rarity::UltraRare => &[
                100_000., 27_380., 8_614., 4_021., 2_303., 1_486., 1_037., 764., 586., 464., 376.,
                311., 262., 223., 193., 168., 148., 131., 117., 105., 95., 86., 79., 72., 66., 61.,
                57., 53., 49., 46., 43., 40., 38., 35., 33., 32., 30., 28., 27., 26., 24., 23.,
                22., 21., 20., 19., 19., 18., 17., 17.,
            ],
        }
    }
}

#[derive(clap::Subcommand, Debug)]
enum Mode {
    /// Calculate the expected number of boxes you need to open to get the item you want
    ExpectedValue,
    /// Calulcate the probability of opening the item you want after opening a number of boxes
    Probability {
        /// The number of boxes you will open
        num_boxes: usize,
    },
    /// Produce a chart (.csv file) that shows the probabilities and expected values of several combinations of starting treasures and additional opened boxes
    Chart {
        /// The maximum number of starting treasures to consider
        max_treasures: usize,
        /// The maximum number of boxes to consider purchasing for probability calculation
        max_boxes: usize,
        /// The csv file to save expected value and probability information to
        out_file: PathBuf,
    },
}

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    mode: Mode,

    /// The rarity of the item you're trying to open
    rarity: Rarity,

    /// The treasure opening that you're on (should be highlighted by the Dota client). Min 1.
    #[arg(default_value = "1")]
    treasure_opening: usize,
}

fn main() {
    let args = Args::parse();

    if args.treasure_opening < 1 {
        println!("Treasure opening must be 1 or greater");
    } else {
        match args.mode {
            Mode::ExpectedValue => {
                let exp = expected_value(&args.rarity, args.treasure_opening);
                println!("{}", exp)
            }
            Mode::Probability { num_boxes } => {
                let prob = probability(&args.rarity, args.treasure_opening, num_boxes);
                println!("{}", prob);
            }
            Mode::Chart {
                max_treasures,
                max_boxes,
                out_file,
            } => {
                chart(args.rarity, max_treasures, max_boxes, &out_file).unwrap();
            }
        }
    }
}

fn expected_value(rarity: &Rarity, treasure_opening: usize) -> f32 {
    // The probability that we make it to this point
    let mut cum_prob = 1.;
    // Expected value
    let mut exp = 0.;
    rarity
        .odds()
        .iter()
        .enumerate()
        .skip(treasure_opening - 1)
        .for_each(|(i, p)| {
            // The probability of the ith chest being the next one we open is the probability of getting to the ith chest
            // times the probability of opening that chest (1 / p)
            let p = 1. / p;
            exp += ((i + 1) - (treasure_opening - 1)) as f32 * cum_prob * p;

            // Then the probability we make it to the next chest is the probability we made it to this chest times the
            // probability we didn't open this chest
            cum_prob *= 1. - p;
        });
    exp += if treasure_opening <= MAX_ODDS {
        cum_prob * (rarity.odds().last().unwrap() + (MAX_ODDS - treasure_opening + 1) as f32)
    } else {
        *rarity.odds().last().unwrap()
    };

    exp
}

fn probability(rarity: &Rarity, treasure_opening: usize, num_boxes: usize) -> f32 {
    rarity
        .odds()
        .iter()
        .chain(std::iter::repeat(rarity.odds().last().unwrap()))
        .skip(treasure_opening - 1)
        .take(num_boxes)
        .scan(1., |cum_prob, p| {
            // The probability of the ith chest being the next one we open is the probability of getting to the ith chest
            // times the probability of opening that chest (1 / p)
            let p = 1. / p;
            let prob = *cum_prob * p;

            // Then the probability we make it to the next chest is the probability we made it to this chest times the
            // probability we didn't open this chest
            *cum_prob *= 1. - p;

            Some(prob)
        })
        .sum()
}

fn chart(
    rarity: Rarity,
    max_treasures: usize,
    max_boxes: usize,
    out: &PathBuf,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = Writer::from_path(out)?;

    wtr.write_record(
        std::iter::repeat(String::new())
            .take(3)
            .chain((1..=max_boxes).map(|n| n.to_string())),
    )?;

    for treasures in 1..=max_treasures {
        let exp = expected_value(&rarity, treasures);
        wtr.write_record(
            [treasures.to_string(), exp.to_string(), String::new()]
                .into_iter()
                .chain(
                    (1..=max_boxes).map(|boxes| probability(&rarity, treasures, boxes).to_string()),
                ),
        )?;
    }
    Ok(())
}
