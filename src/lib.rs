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
    let mut ledger: Vec<Line> = Vec::with_capacity(input.lines().count());

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
        match &line.kind {
            LineKind::Posting(tokens) => {
                max_acct_len = max_acct_len
                    .max(tokens[0].chars().count() + if has_status(&tokens[0]) { 2 } else { 4 });
                // + 2 spaces between account and amount
                max_line_len = max_line_len.max(
                    max_acct_len
                        + match tokens.len() >= 2 {
                            true => format_amount(&tokens[1])?,
                            false => "".to_string(),
                        }
                        .chars()
                        .count()
                        + 2,
                );
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
            LineKind::Posting(tokens) => {
                line.content = Some(format!(
                    "{:max_acct_len$}  {}",
                    format_account(&tokens[0]),
                    match tokens.len() >= 2 {
                        true => format_amount(&tokens[1])?,
                        false => "".to_string(),
                    }
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

    // Check for blank line to avoid error when getting first character
    if line.content.as_ref().unwrap().trim().is_empty() {
        line.kind = LineKind::Other;
        return line;
    }

    line.kind = match line.content.as_ref().unwrap().chars().next().unwrap() {
        '0'..='9' => LineKind::Date,
        ' ' | '\t' => LineKind::Posting(tokenize(&line)),
        _ => LineKind::Other,
    };

    line
}

/// This does not determine if the amount is a valid number.
/// It only assesses whether it is comprised of chars that compose a number.
fn format_amount(token: &str) -> Result<String, String> {
    let mut output = Amount {
        amount: 0,
        precision: 0,
        currency: None,
    };

    let mut number_index = None;
    let mut currency_index = None;
    for (i, char) in token.chars().enumerate() {
        if is_number_component(char) {
            if number_index.is_none() {
                number_index = Some(i);
            }
        } else if currency_index.is_none() && char != ' ' {
            currency_index = Some(i);
        }
        if number_index.is_some() && currency_index.is_some() {
            break;
        }
    }

    let number_index = match number_index {
        Some(i) => i,
        None => return Err(format!("No amount found in token: {token}")),
    };

    let amount: String;
    if let Some(currency_index) = currency_index {
        let prepend_currency = currency_index < number_index;
        output.currency = Some(Currency {
            symbol: if prepend_currency {
                amount = token[number_index..].to_string();
                token[currency_index..number_index].trim().to_string()
            } else {
                amount = token[number_index..currency_index].to_string();
                token[currency_index..].trim().to_string()
            },
            prepend: prepend_currency,
        });
    } else {
        amount = token.to_string();
    }

    let amount = amount.replace(',', "").trim().to_string();
    output.precision = amount.len() - amount.find('.').unwrap_or(amount.len() - 1) - 1;

    output.amount = match amount.replace('.', "").parse::<isize>() {
        Ok(number) => number,
        Err(_) => return Err(format!("Cannot parse `isize` in amount: {amount}")),
    };

    Ok(output.to_string())
}

fn format_account(account: &str) -> String {
    if has_status(account) { "  " } else { "    " }.to_string() + account.trim()
}

fn is_number_component(char: char) -> bool {
    char.is_ascii_digit() || char == '-' || char == '.' || char == ','
}

fn has_status(token: &str) -> bool {
    token.starts_with("! ") || token.starts_with("* ")
}
