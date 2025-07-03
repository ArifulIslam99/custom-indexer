// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use sui_types::full_checkpoint_content::CheckpointData;
use serde_json;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

#[tokio::main]
async fn main() -> Result<()> {
    // Define input and output directories
    let checkpoint_dir = "/tmp/checkpoints";
    let json_dir = "/tmp/checkpoints_json";

    // Create output directory if it doesn't exist
    fs::create_dir_all(json_dir)?;

    // Iterate over .chk files in checkpoint_dir
    let dir = fs::read_dir(checkpoint_dir)?;
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "chk") {
            // Read checkpoint file
            let mut file = File::open(&path)?;
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer)?;

            // Deserialize BCS data into CheckpointData
            let checkpoint: CheckpointData = bcs::from_bytes(&buffer)?;

            // Define output JSON file path (e.g., 162000013.chk -> 162000013.json)
            let file_name = path.file_stem().unwrap().to_str().unwrap();
            let json_file = Path::new(json_dir).join(format!("{}.json", file_name));

            // Convert to JSON
            let json_output = serde_json::to_string_pretty(&checkpoint)?;

            // Save JSON to file
            let mut json_file_handle = AsyncFile::create(&json_file).await?;
            json_file_handle.write_all(json_output.as_bytes()).await?;
            println!("Saved JSON to {}", json_file.display());
        }
    }

    Ok(())
}