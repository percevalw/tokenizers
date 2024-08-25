use serde::{Deserialize, Serialize};
use regex::Regex;
use crate::normalizer::Range;
use crate::tokenizer::{
    pattern::Invert, PreTokenizedString, PreTokenizer, Result, SplitDelimiterBehavior,
};
use crate::utils::macro_rules_attribute;

#[derive(Clone, Debug, PartialEq, Eq)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct Whitespace;

impl Default for Whitespace {
    fn default() -> Self {
        Self
    }
}

impl PreTokenizer for Whitespace {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        lazy_static! {
            static ref RE: Regex = Regex::new(r"\w+|[^\w\s]+").unwrap();
        }
        let re_ref: &Regex = &RE;

        pretokenized.split(|_, normalized| {
            normalized.split(Invert(re_ref), SplitDelimiterBehavior::Removed)
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct WhitespaceSplit;

impl PreTokenizer for WhitespaceSplit {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        pretokenized.split(|_, normalized| {
            normalized.split(char::is_whitespace, SplitDelimiterBehavior::Removed)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq)]
pub enum EditBoundariesBehavior {
    None,
    EnsureSpace,
    StripSpace,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct EditBoundaries {
    #[serde(default = "default_none")]
    left: EditBoundariesBehavior,
    #[serde(default = "default_none")]
    right: EditBoundariesBehavior,
}

fn default_none() -> EditBoundariesBehavior { EditBoundariesBehavior::None }

impl EditBoundaries {
    pub fn new(left: EditBoundariesBehavior, right: EditBoundariesBehavior) -> Self {
        Self { left, right }
    }
}


impl PreTokenizer for EditBoundaries {
    fn pre_tokenize(&self, pretokenized: &mut PreTokenizedString) -> Result<()> {
        pretokenized.split(|_, mut normalized| {
            // Just remove the first character if it's whitespace
            let mut left_delta: isize = 0;
            let mut right_delta: isize = 0;
            if self.left == EditBoundariesBehavior::EnsureSpace && normalized.get().chars().nth(0).unwrap() != ' ' {
                left_delta -= 1;
            } else if self.left == EditBoundariesBehavior::StripSpace && normalized.get().chars().nth(0).unwrap() == ' ' {
                left_delta += 1;
            }
            if self.right == EditBoundariesBehavior::EnsureSpace && normalized.get().chars().nth_back(0).unwrap() != ' ' {
                right_delta += 1;
            } else if self.right == EditBoundariesBehavior::StripSpace && normalized.get().chars().nth_back(0).unwrap() == ' ' {
                right_delta -= 1;
            }
            if left_delta == 0 && right_delta == 0 {
                return Ok(vec![normalized]);
            }
            if left_delta >= 0 && right_delta <= 0 {
                if normalized.len() < (left_delta - right_delta) as usize {
                    normalized = normalized.slice(Range::Normalized(0..0)).unwrap();
                } else {
                    normalized = normalized.slice(Range::Normalized(left_delta as usize..(normalized.len() as isize + right_delta) as usize)).unwrap();
                }
            }
            else {
                if left_delta < 0 {
                    normalized.prepend(" ");
                }
                if right_delta > 0 {
                    normalized.append(" ");
                }
            }
            // Prepend a space if needed
            Ok(vec![normalized])
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OffsetReferential, OffsetType, PreTokenizer};

    #[test]
    fn basic() {
        let tests = vec![
            (
                "Hey man!",
                vec![("Hey", (0, 3)), ("man", (4, 7)), ("!", (7, 8))],
            ),
            (
                "How are you doing?",
                vec![
                    ("How", (0, 3)),
                    ("are", (4, 7)),
                    ("you", (8, 11)),
                    ("doing", (12, 17)),
                    ("?", (17, 18)),
                ],
            ),
            ("\n", vec![]),
        ];
        let pretok = Whitespace {};
        for (s, res) in tests {
            let mut pretokenized = PreTokenizedString::from(s);
            pretok.pre_tokenize(&mut pretokenized).unwrap();
            assert_eq!(
                pretokenized
                    .get_splits(OffsetReferential::Original, OffsetType::Byte)
                    .into_iter()
                    .map(|(s, o, _)| (s, o))
                    .collect::<Vec<_>>(),
                res
            );
        }
    }

    #[test]
    fn whitespace_split() {
        let tests = vec![
            ("Hey man!", vec![("Hey", (0, 3)), ("man!", (4, 8))]),
            (
                "Hey, man, Good?",
                vec![("Hey,", (0, 4)), ("man,", (5, 9)), ("Good?", (10, 15))],
            ),
        ];
        let pretok = WhitespaceSplit;
        for (s, res) in tests {
            let mut pretokenized = PreTokenizedString::from(s);
            pretok.pre_tokenize(&mut pretokenized).unwrap();
            assert_eq!(
                pretokenized
                    .get_splits(OffsetReferential::Original, OffsetType::Byte)
                    .into_iter()
                    .map(|(s, o, _)| (s, o))
                    .collect::<Vec<_>>(),
                res
            );
        }
    }
}
