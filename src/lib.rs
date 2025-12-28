/*
 * ledgible - Formatter for ledger hledger journals
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

use std::fmt::Write;

enum LineKind {
    Date,
    Posting,
    Comment,
    Other,
    None,
}

struct Line {
    kind: LineKind,
    content: Option<String>,
    comment: Option<String>,
}

pub fn format(buffer: &mut String, input: &str) -> Result<(), std::fmt::Error> {
    let ledger: Vec<Line> = input.lines().map(process).collect();

    // Determine proper spacing
    let mut max_acct_len = 0;
    let mut max_line_len = 0;
    for line in &ledger {
        match line.kind {
            LineKind::Posting => {
                let tokens = tokenize(line);
                max_acct_len = max_acct_len
                    .max(tokens[0].chars().count() + if has_status(&tokens[0]) { 2 } else { 4 });
                // + 2 spaces between account and amount
                max_line_len =
                    max_line_len.max(max_acct_len + format_amount(&tokens[1]).chars().count() + 2);
            }
            LineKind::Comment => (),
            _ => max_line_len = max_line_len.max(line.content.as_ref().unwrap().chars().count()),
        }
    }

    // write cycle
    for mut line in ledger {
        match line.kind {
            LineKind::Comment => {
                writeln!(buffer, "{}", line.comment.unwrap())?;
                continue;
            }
            LineKind::Posting => {
                let tokens = tokenize(&line);
                line.content = Some(format!(
                    "{:max_acct_len$}  {}",
                    format_account(&tokens[0]),
                    format_amount(&tokens[1])
                ));
            }
            LineKind::None => panic!("Line kind undefined"),
            _ => (),
        }

        writeln!(
            buffer,
            "{}",
            match line.comment {
                Some(comment) => format!("{:max_line_len$} ;{}", line.content.unwrap(), comment),
                None => format!("{:max_line_len$}", line.content.unwrap()),
            }
            .trim_end()
        )?;
    }
    buffer.truncate(buffer.trim_end().len());
    buffer.push('\n');
    Ok(())
}

fn tokenize(line: &Line) -> Vec<String> {
    match &line.content {
        Some(content) => content
            .split("  ")
            .map(|x| x.trim().to_string())
            .filter(|x| !x.is_empty())
            .collect(),
        None => vec![],
    }
}

// TODO: make more efficient
fn process(data: &str) -> Line {
    let mut line: Line;
    if data.trim_start().starts_with([';', '#']) {
        return Line {
            kind: LineKind::Comment,
            content: None,
            comment: Some(data.trim_end().to_string()),
        };
    }

    line = match data.split_once([';', '#']) {
        None => Line {
            kind: LineKind::None,
            content: Some(data.trim_end().to_string()),
            comment: None,
        },
        Some((data, comment)) => Line {
            kind: LineKind::None,
            content: Some(data.trim_end().to_string()),
            comment: Some(comment.trim_end().to_string()),
        },
    };

    // Determine line type
    let tokens = tokenize(&line);

    // Check for blank line to avoid panic when getting first character
    if tokens.is_empty() {
        line.kind = LineKind::Other;
        return line;
    }

    let first_char = line.content.as_ref().unwrap().chars().next().unwrap();
    if tokens.len() == 1 && tokens[0].chars().next().unwrap().is_ascii_digit() {
        line.kind = LineKind::Date;
    } else if !tokens.is_empty() && (first_char == ' ' || first_char == '\t') {
        // only postings will start with whitespace
        line.kind = LineKind::Posting;
    } else {
        line.kind = LineKind::Other;
    }

    line
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

    {
        if currency.is_empty() {
            number.to_string()
        } else if currency_prefix {
            match number.chars().next().unwrap() {
                '-' => format!("{currency}{number}"),
                _ => format!("{currency} {number}"),
            }
        } else {
            format!("{number} {currency}")
        }
    }
    .trim_end()
    .to_string()
}

fn format_account(account: &str) -> String {
    if has_status(account) { "  " } else { "    " }.to_string() + account.trim()
}

fn is_number_component(char: char) -> bool {
    char.is_ascii_digit() || char == '-' || char == '.' || char == ','
}

fn has_status(token: &str) -> bool {
    let mut chars = token.chars();
    match chars.next().unwrap() {
        '!' | '*' => chars.next().unwrap().is_ascii_whitespace(),
        _ => false,
    }
}
