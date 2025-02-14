use anyhow::Result;
use std::{fs, path::PathBuf};

pub fn read_batch_quotes(path: PathBuf) -> Result<Vec<Vec<u8>>> {
    let mut batch = vec![];

    let batch_str = fs::read_to_string(path)?;
    for quote in batch_str.split(',') {
        batch.push(hex::decode(quote[2..].trim())?);
    }

    Ok(batch)
}
