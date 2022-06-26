mod solutions;
mod words;

use crate::solutions::SOLUTIONS;
use crate::words::WORDS;
use miette::{miette, IntoDiagnostic};
use std::collections::HashSet;
use std::path::Iter;

fn main() {}

#[derive(Debug)]
struct Input {
    solution: String,
    guesses: Vec<Guess>,
}

impl Input {
    fn from_emoji(emoji: &str) -> miette::Result<Self> {
        let mut lines = emoji.lines();

        // Wordle [release] [attempts]
        let mut line = lines
            .next()
            .ok_or(miette!("Missing first line"))?
            .split(" ");
        let wordle = line.next().unwrap();
        let release: usize = line.next().unwrap().parse().into_diagnostic()?;
        let attempts: usize = line
            .next()
            .unwrap()
            .split("/")
            .next()
            .unwrap()
            .parse()
            .into_diagnostic()?;

        let solution = SOLUTIONS[release].to_string();

        // Blank line
        lines.next().ok_or(miette!("Missing blank line"))?;

        // Guesses
        let mut guesses = Vec::new();
        for _ in 0..attempts {
            let line = lines.next().ok_or(miette!("Missing guess line"))?;
            let guess = Guess::from_emoji(line)?;
            guesses.push(guess);
        }

        Ok(Input { solution, guesses })
    }

    /// Gets the char at position.
    pub fn letter(&self, idx: usize) -> char {
        self.solution.chars().nth(idx).unwrap()
    }

    /// Returns a HashSet of chars that aren't in that position.
    pub fn other_letters(&self, idx: usize) -> HashSet<char> {
        self.solution
            .chars()
            .enumerate()
            .filter_map(|(i, c)| if idx == i { None } else { Some(c) })
            .collect()
    }

    /// Returns all chars from the soltuion as a HashSet.
    pub fn all_letters(&self) -> HashSet<char> {
        self.solution.chars().collect()
    }
}

#[derive(Debug)]
struct Guess {
    letters: Vec<Colour>,
}

impl Guess {
    fn from_emoji(emoji: &str) -> miette::Result<Self> {
        let letters = emoji
            .chars()
            .map(Colour::from_char)
            .collect::<miette::Result<Vec<Colour>, miette::Error>>()?;

        Ok(Guess { letters })
    }
}

#[derive(Debug)]
enum Colour {
    Green,
    Yellow,
    Grey,
}

impl Colour {
    fn from_char(c: char) -> miette::Result<Self> {
        match c {
            '🟩' => Ok(Colour::Green),
            '🟨' => Ok(Colour::Yellow),
            '⬛' => Ok(Colour::Grey),
            _ => Err(miette!("Unknown emoji: {}", c)),
        }
    }
}

fn find(letters: Vec<PossibleLetters>) -> Vec<&'static str> {
    let mut found = Vec::with_capacity(2048);
    // dbg!(&letters);

    for word in WORDS {
        let matches = letters.iter().zip(word.chars()).all(|(l, c)| match l {
            PossibleLetters::Any => true,
            PossibleLetters::Exact(o) => c == *o,
            PossibleLetters::Possible(chars) => chars.contains(&c),
            PossibleLetters::Not(chars) => !chars.contains(&c),
        });
        if matches {
            found.push(*word);
        }
    }
    found
}

trait Solver {
    fn reverdle(input: &Input) -> Vec<&str>;
}

#[derive(Debug)]
enum PossibleLetters {
    Any,
    Exact(char),
    Possible(HashSet<char>),
    Not(HashSet<char>),
}

/// Only looks at the first guess.
struct Naive;

impl Solver for Naive {
    fn reverdle(input: &Input) -> Vec<&str> {
        let first_guess = &input.guesses[0];
        let possibles = first_guess
            .letters
            .iter()
            .enumerate()
            .map(|(idx, letter)| match letter {
                Colour::Green => PossibleLetters::Exact(input.letter(idx)),
                Colour::Yellow => PossibleLetters::Possible(input.other_letters(idx)),
                Colour::Grey => PossibleLetters::Not(input.all_letters()),
            })
            .collect::<Vec<_>>();

        find(possibles)
    }
}

/// Starts from the second last guess and works upwards recursively.
struct Recursive;

impl Recursive {
    fn recurse(input: &Input, level: usize, bad_chars: HashSet<char>) -> Vec<&str> {
        let guess = &input.guesses[level];
        let possibles = guess
            .letters
            .iter()
            .enumerate()
            .map(|(idx, letter)| match letter {
                Colour::Green => PossibleLetters::Exact(input.letter(idx)),
                Colour::Yellow => PossibleLetters::Possible(
                    input
                        .other_letters(idx)
                        .difference(&bad_chars)
                        .map(|c| *c)
                        .collect(),
                ),
                Colour::Grey => {
                    PossibleLetters::Not(input.all_letters().union(&bad_chars).cloned().collect())
                }
            })
            .collect::<Vec<_>>();

        dbg!(&possibles);
        let found = find(possibles);

        println!(
            "level: {} guess: {:?} bad: {:?} found: {} {:?}",
            level,
            guess,
            &bad_chars,
            found.len(),
            found,
        );

        return if level == 0 {
            found
        } else {
            let mut new_set = HashSet::with_capacity(2048);
            for word in found {
                let mut new_bad_chars = bad_chars.clone();
                for c in word.chars() {
                    new_bad_chars.insert(c);
                }
                let found = Self::recurse(input, level - 1, new_bad_chars);
                let found: HashSet<&str> = HashSet::from_iter(found);
                new_set = new_set.union(&found).map(|c| *c).collect::<HashSet<&str>>();
            }
            new_set.into_iter().collect()
        };
    }
}

impl Solver for Recursive {
    fn reverdle(input: &Input) -> Vec<&str> {
        let level = input.guesses.len() - 2;
        Self::recurse(input, level, HashSet::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let input = Input::from_emoji(include_str!("../fixtures/371-gak.txt")).unwrap();
        dbg!(&input);
        let iter = Recursive::reverdle(&input);
        println!("Iterative: {:?}", iter);
        let naive = Naive::reverdle(&input);
        println!("Naive: {:?}", naive);

        println!("Iterative: {}", iter.len());
        println!("Naive: {}", naive.len());
    }
}
