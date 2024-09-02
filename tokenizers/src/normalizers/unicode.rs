use std::collections::HashMap;
use std::iter::FromIterator;
use crate::tokenizer::{NormalizedString, Normalizer, Result};
use crate::utils::{macro_rules_attribute, SysRegex};
use any_ascii::{any_ascii_char};
use serde::{Deserialize, Deserializer, Serialize};
use crate::normalizer::Range;
use crate::pre_tokenizers::split::{Split, SplitPattern};
use crate::SplitDelimiterBehavior;

#[derive(Default, Copy, Clone, Debug)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct NFD;
impl Normalizer for NFD {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        normalized.nfd();
        Ok(())
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct NFKD;
impl Normalizer for NFKD {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        normalized.nfkd();
        Ok(())
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct NFC;
impl Normalizer for NFC {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        normalized.nfc();
        Ok(())
    }
}

#[derive(Default, Copy, Clone, Debug)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct NFKC;
impl Normalizer for NFKC {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        normalized.nfkc();
        Ok(())
    }
}

/**
This normalizer converts all characters that are not part of the ASCII set.
Only chars in a user-defined hashmap are kept.
*/
#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub struct AnyASCII {
    kept_pattern: Option<String>,
    #[serde(skip)]
    regex: Option<SysRegex>,
    char_map: HashMap<char, String>,
}

impl Clone for AnyASCII {
    fn clone(&self) -> Self {
        Self::new(
            self.kept_pattern.clone(),
            Some(self.char_map.clone()),
        ).unwrap()
    }
}


impl AnyASCII {
    pub fn new(
        kept_pattern: Option<String>,
        char_map: Option<HashMap<char, String>>,
    ) -> Result<Self> {
        let regex = match &kept_pattern {
            Some(pattern) => Some(SysRegex::new(pattern)?),
            None => None,
        };
        Ok(Self {
            kept_pattern,
            regex,
            char_map: char_map.unwrap_or_default(),
        })
    }
}

impl<'de> Deserialize<'de> for AnyASCII {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        enum Type {
            AnyASCII,
        }

        #[derive(Deserialize)]
        pub struct AnyASCIIHelper {
            #[serde(rename = "type")]
            _type: Type,
            kept_pattern: Option<String>,
            char_map: HashMap<char, String>,
        }

        let helper = AnyASCIIHelper::deserialize(deserializer)?;
        Self::new(helper.kept_pattern, Some(helper.char_map)).map_err(serde::de::Error::custom)
    }
}


impl Normalizer for AnyASCII {
    fn normalize(&self, string: &mut NormalizedString) -> Result<()> {
        let mut last_offset = 0;
        let mut transformations: Vec<(char, isize)> = vec![];
        if let Some(regex) = &self.regex {
            regex.find_iter(
                string.get()
            )
                .for_each(|(start, end)| {
                    if start > last_offset {
                        transformations.extend(
                            string.get_range(Range::Normalized(last_offset..start))
                                .unwrap()
                                .chars()
                                .map(|c: char| {
                                    // First lookup the char map
                                    let replacement = match self.char_map.get(&c) {
                                        Some(replacement) => replacement.clone(),
                                        None => {
                                            if c.is_ascii() {
                                                return vec![(c, 0)];
                                            }
                                            any_ascii_char(c).to_string()
                                        }
                                    };
                                    return
                                        replacement
                                            .chars()
                                            .enumerate()
                                            .map(|(i, new_c)| {
                                                if i == 0 {
                                                    (new_c, 0)
                                                } else {
                                                    (new_c, 1)
                                                }
                                            })
                                            .collect::<Vec<(char, isize)>>();
                                }).flatten()
                        );
                    }
                    transformations.extend(
                        string.get_range(Range::Normalized(start..end))
                            .unwrap().chars()
                            .map(|c| {
                                (c, 0)
                            })
                    );
                    last_offset = end;
                });
        }
        if last_offset < string.len() {
            transformations.extend(
                string.get_range(Range::Normalized(last_offset..))
                    .unwrap()
                    .chars()
                    .map(|c: char| {
                        // First lookup the char map
                        let replacement = match self.char_map.get(&c) {
                            Some(replacement) => replacement.clone(),
                            None => {
                                if c.is_ascii() {
                                    return vec![(c, 0)];
                                }
                                any_ascii_char(c).to_string()
                            }
                        };
                        return
                            replacement
                                .chars()
                                .enumerate()
                                .map(|(i, new_c)| {
                                    if i == 0 {
                                        (new_c, 0)
                                    } else {
                                        (new_c, 1)
                                    }
                                })
                                .collect::<Vec<(char, isize)>>();
                    }).flatten()
            );
        }
        string.transform(transformations, 0);
        Ok(())
    }
}

fn do_nmt(normalized: &mut NormalizedString) {
    // Ascii Control characters
    normalized
        .filter(|c| {
            !matches!(
                c as u32,
                0x0001..=0x0008 |
                0x000B |
                0x000E..=0x001F |
                0x007F |
                0x008F |
                0x009F
            )
        })
        // Other code points considered as whitespace.
        .map(|c| match c as u32 {
            0x0009 => ' ',
            0x000A => ' ',
            0x000C => ' ',
            0x000D => ' ',
            0x1680 => ' ',
            0x200B..=0x200F => ' ',
            0x2028 => ' ',
            0x2029 => ' ',
            0x2581 => ' ',
            0xFEFF => ' ',
            0xFFFD => ' ',
            _ => c,
        });
}

#[derive(Default, Copy, Clone, Debug)]
#[macro_rules_attribute(impl_serde_type!)]
pub struct Nmt;
impl Normalizer for Nmt {
    fn normalize(&self, normalized: &mut NormalizedString) -> Result<()> {
        do_nmt(normalized);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfkc() {
        let original = "\u{fb01}".to_string();
        let normalized = "fi".to_string();
        let mut n = NormalizedString::from(original.clone());
        NFKC.normalize(&mut n).unwrap();

        assert_eq!(
            n,
            NormalizedString::new(original, normalized, vec![(0, 3), (0, 3)], 0)
        );

        assert_eq!(n.alignments_original(), vec![(0, 2), (0, 2), (0, 2)]);
    }
}
