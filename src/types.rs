/*
 * ledgible - Formatter for ledger and hledger journals
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

pub struct Line {
    pub kind: LineKind,
    pub content: Option<String>,
    pub comment: Option<Comment>,
}

pub struct Comment {
    pub delimiter: Delimiter,
    pub content: String,
}

pub struct Amount {
    pub amount: isize,
    pub precision: usize,
    pub currency: Option<Currency>,
}

pub struct Currency {
    pub symbol: String,
    pub prepend: bool,
}

pub enum LineKind {
    Date,
    Posting(Vec<String>),
    Comment,
    Other,
    None,
}

pub enum Delimiter {
    Hash,
    Semicolon,
    None,
}

impl std::fmt::Display for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{}",
            match self.delimiter {
                Delimiter::Hash => "#",
                Delimiter::Semicolon => ";",
                Delimiter::None => "",
            },
            self.content
        )
    }
}

impl std::fmt::Display for Amount {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut out: String = format!("{:.*}", self.precision, self.amount);
        if self.precision > 0 {
            out.insert(out.len() - self.precision, '.');
        }
        if let Some(currency) = &self.currency {
            match currency.prepend {
                true => {
                    if self.amount >= 0 {
                        out.insert(0, ' ');
                    }
                    out.insert_str(0, &currency.symbol);
                }
                false => out.push_str(&currency.symbol),
            }
        }
        write!(f, "{}", out)
    }
}
