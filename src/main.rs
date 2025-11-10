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

use std::io::{BufRead, BufReader};

fn main() -> std::io::Result<()> {
    let ledger = std::fs::read_to_string(match std::env::var("LEDGER_FILE") {
        Ok(val) => val,
        Err(_) => String::from("~/.hledger.journal"),
    })?;

    let mut max_acct_len = 0;
    let mut max_line_len = 0;
    for line in ledger.lines() {
        let tokens = tokenize(line);

        if tokens.len() == 2 {
            // crude, but assume split
            max_acct_len = max_acct_len.max(tokens[0].chars().count()); // len() != length
            // 4 from indent + 2 between account and amount
            max_line_len = max_line_len.max(max_acct_len + tokens[1].chars().count() + 6);
        } else {
            max_line_len = max_line_len.max(strip_comments(line).chars().count());
        }
    }

    // write cycle
    for line in ledger.lines() {
        if line.trim_start().starts_with(";") {
            println!("{line}");
            continue;
        }

        let tokens = tokenize(line);

        println!(
            "{:max_line_len$}{}",
            match tokens.len() {
                2 => format!("    {:max_acct_len$}  {}", tokens[0], tokens[1]),
                _ => strip_comments(line).to_string(),
            },
            comments(line)
        );
    }

    Ok(())
}

fn tokenize(line: &str) -> Vec<String> {
    strip_comments(line)
        .trim_start()
        .split("  ")
        .filter(|x| !x.is_empty())
        .map(|x| x.trim().to_string())
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
