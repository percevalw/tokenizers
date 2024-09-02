use crate::tokenizer::{Model, Result, Token};
use crate::{AddedToken, Trainer};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
// Re-export

#[derive(PartialEq, Clone, Eq, Default, Serialize, Deserialize)]
pub struct Noop {}

impl std::fmt::Debug for Noop {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt.debug_struct("Noop").finish()
    }
}

impl Noop {}

/// A `NoopTrainer` doesn't do anything, it's just here for consistency
#[non_exhaustive]
#[derive(Builder, Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoopTrainer {}

impl Trainer for NoopTrainer {
    type Model = Noop;

    fn train(&self, model: &mut Noop) -> Result<Vec<AddedToken>> {
        Ok(vec![])
    }

    fn should_show_progress(&self) -> bool {
        false
    }

    fn feed<I, S, F>(&mut self, iterator: I, process: F) -> Result<()>
    where
        I: Iterator<Item=S> + Send,
        S: AsRef<str> + Send,
        F: Fn(&str) -> Result<Vec<String>> + Sync,
    {
        Ok(())
    }
}

impl Model for Noop {
    type Trainer = NoopTrainer;

    fn tokenize(&self, token: &str) -> Result<Vec<Token>> {
        Ok(vec![Token {
            id: 0,
            value: token.to_owned(),
            offsets: (0, token.len()),
        }])
    }

    fn token_to_id(&self, token: &str) -> Option<u32> { None }

    fn id_to_token(&self, id: u32) -> Option<String> {
        None
    }

    fn get_vocab(&self) -> HashMap<String, u32> { HashMap::new() }

    fn get_vocab_size(&self) -> usize {
        0
    }

    fn save(&self, _folder: &Path, _name: Option<&str>) -> Result<Vec<PathBuf>> {
        Ok(vec![])
    }

    fn get_trainer(&self) -> <Self as Model>::Trainer {
        NoopTrainer {}
    }
}

#[cfg(test)]
mod tests {}
