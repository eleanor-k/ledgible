/*
 * ledgible - Formatter for hledger journals
 * Copyright (C) 2025  Eleanor Kelley <me at eleanorkelley dot com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

use clap::{Arg, command, error::ErrorKind};
use std::{
    fmt::Write,
    fs::write,
    io::{Read, stdin},
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = command!()
        .arg(Arg::new("input").value_name("FILE").help("Input journal"))
        .arg(
            Arg::new("output")
                .short('o')
                .long("output")
                .value_name("FILE")
                .help("Write formatted journal to file"),
        )
        .arg(
            Arg::new("overwrite")
                .short('i')
                .long("inplace")
                .requires("input")
                .conflicts_with("output")
                .action(clap::ArgAction::SetTrue)
                .help("Overwrite input file"),
        );
    let matches = cmd.get_matches_mut();

    // Read appropriate input into `ledger`
    let mut ledger = String::new();
    let input = matches.get_one::<String>("input");
    if let Some(file) = input {
        let read = std::fs::read_to_string(file);
        match read {
            Ok(file) => ledger = file,
            Err(_) => cmd.error(ErrorKind::Io, "Cannot read file").exit(),
        }
    } else {
        stdin().read_to_string(&mut ledger)?;
    }

    let mut max_acct_len = 0;
    let mut max_line_len = 0;
    for line in ledger.lines() {
        let tokens = tokenize(line);

        if tokens.len() == 2 {
            // crude, but assume split
            max_acct_len = max_acct_len.max(tokens[0].chars().count()); // len() != length
            // 4 from indent + 2 between account and amount
            max_line_len =
                max_line_len.max(max_acct_len + format_amount(&tokens[1]).chars().count() + 6);
        } else {
            max_line_len = max_line_len.max(strip_comments(line).chars().count());
        }
    }

    // write cycle
    let mut buffer = String::new();
    for line in ledger.lines() {
        if line.trim_start().starts_with(";") {
            writeln!(&mut buffer, "{line}")?;
            continue;
        }

        let tokens = tokenize(line);

        writeln!(
            &mut buffer,
            "{:max_line_len$}{}",
            match tokens.len() {
                2 => format!(
                    "    {:max_acct_len$}  {}",
                    tokens[0],
                    format_amount(&tokens[1])
                ),
                _ => strip_comments(line),
            },
            comments(line)
        )?;
    }

    // Write output
    match matches.get_one::<String>("output") {
        Some(file) => write(file, buffer)?,
        None => match matches.get_flag("overwrite") {
            true => {
                let tempfile = format!("{}.old", input.unwrap());
                write(&tempfile, buffer)?;
                std::fs::rename(tempfile, input.unwrap())?
            }
            false => print!("{buffer}"),
        },
    };

    Ok(())
}

fn tokenize(line: &str) -> Vec<String> {
    strip_comments(line)
        .split("  ")
        .map(|x| x.trim().to_string())
        .filter(|x| !x.is_empty())
        .collect()
}

fn comments(line: &str) -> String {
    if line.trim_start().starts_with(";") {
        return line.to_string();
    }

    match line.split_once(";") {
        None => String::from(""),
        Some((_, x)) => format!(" ;{x}"),
    }
}

fn strip_comments(line: &str) -> String {
    match line.split_once(";") {
        None => line.to_string(),
        Some((x, _)) => x.trim_end().to_string(),
    }
}

/// This does not determine if the amount is a valid number.
/// It only assesses whether it is comprised of chars that compose a number.
fn format_amount(token: &str) -> String {
    let currency_prefix = !is_number_component(token.chars().next().unwrap());
    let mut number = String::new();
    let mut currency = String::new();

    let mut is_number = false;
    let mut is_currency = false;

    // this loop could probably be refactored
    for char in token.chars() {
        if is_number {
            if is_number_component(char) {
                number.push(char);
                continue;
            } else {
                is_number = false;
            }
        } else if is_currency {
            if char == ' ' || is_number_component(char) {
                is_currency = false;
            } else {
                currency.push(char);
                continue;
            }
        }

        // if we're not building a currency or number, start building
        if is_number_component(char) {
            assert!(number.is_empty());
            is_number = true;
            number.push(char);
        } else if char != ' ' {
            // is not part of a number; should be currency
            assert!(currency.is_empty());
            is_currency = true;
            currency.push(char);
        }
    }

    if currency.is_empty() {
        number
    } else if currency_prefix {
        match number.chars().next().unwrap() {
            '-' => format!("{currency}{number}"),
            _ => format!("{currency} {number}"),
        }
    } else {
        format!("{number} {currency}")
    }
}

fn is_number_component(char: char) -> bool {
    char.is_ascii_digit() || char == '-' || char == '.' || char == ','
}
