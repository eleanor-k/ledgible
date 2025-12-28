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

mod types;

use crate::types::*;
use std::fmt::Write;

// TODO: Streamline logic
pub fn format(buffer: &mut String, input: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut ledger: Vec<Line> = Vec::new();

    let mut comment_block: Option<usize> = None;
    for (i, line) in input
        .lines()
        .enumerate()
        .skip_while(|(_, line)| line.trim().is_empty())
    {
        let i = i + 1; // Reassign to ensure line numbers make sense

        let mut kind = match comment_block {
            Some(_) => LineKind::Comment,
            None => LineKind::None,
        };

        match line.trim().to_lowercase().as_str() {
            "comment" => {
                if comment_block.is_none() {
                    comment_block = Some(i); // For error printing later
                    kind = LineKind::Comment;
                }
            }
            "end comment" => {
                match comment_block {
                    Some(_) => comment_block = None,
                    None => {
                        return Err(format!("Unexpected `{}` at line {}", line.trim(), i).into());
                    }
                };
            }
            _ => (),
        }

        // Parse raw line
        ledger.push(assign_kind(Line {
            kind,
            content: Some(line.to_string()),
            comment: None,
        }));
    }

    // Check if there's a dangling comment block
    if let Some(i) = comment_block {
        return Err(format!("Unenclosed comment block at line {}", i).into());
    }

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
                writeln!(buffer, "{}", line.comment.unwrap().content)?;
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
            LineKind::None => return Err("Line kind undefined".into()),
            _ => (),
        }

        writeln!(
            buffer,
            "{}",
            match line.comment {
                Some(comment) => format!("{:max_line_len$} {}", line.content.unwrap(), comment),
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

fn parse_comments(line: &mut Line) {
    let content = line.content.take().unwrap();
    if matches!(line.kind, LineKind::Comment) || content.trim_start().starts_with([';', '#']) {
        line.kind = LineKind::Comment;
        line.content = None;
        line.comment = Some(Comment {
            // This can probably be handled better
            delimiter: Delimiter::None,
            content: content.trim_end().to_string(),
        });
    } else {
        match content.find([';', '#']) {
            Some(index) => {
                let delimiter = match content.chars().nth(index).unwrap() {
                    ';' => Delimiter::Semicolon,
                    '#' => Delimiter::Hash,
                    _ => unreachable!(),
                };

                let (content, comment) = content.split_at(index);
                line.content = Some(content.trim_end().to_string());
                line.comment = Some(Comment {
                    delimiter,
                    content: comment[1..].trim_end().to_string(),
                });
            }
            None => {
                line.content = Some(content.trim_end().to_string());
                line.comment = None;
            }
        }
    }
}

// TODO: make more efficient
fn assign_kind(mut line: Line) -> Line {
    parse_comments(&mut line);
    if let LineKind::Comment = line.kind {
        return line;
    }

    // Determine line type
    let tokens = tokenize(&line);

    // Check for blank line to avoid error when getting first character
    if tokens.is_empty() {
        line.kind = LineKind::Other;
        return line;
    }

    let first_char = line.content.as_ref().unwrap().chars().next().unwrap();
    if tokens[0].chars().next().unwrap().is_ascii_digit() {
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
        '!' | '*' => chars.next().unwrap() == ' ',
        _ => false,
    }
}
