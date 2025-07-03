// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{self, File};
use std::io::Read;
use std::path::Path;
use tokio::fs::File as AsyncFile;
use tokio::io::AsyncWriteExt;

// Struct to hold filtered transaction data
#[derive(Serialize, Deserialize)]
struct FilteredTransaction {
    checkpoint_sequence: u64,
    transaction: Value,
    effects: Value,
    events: Option<Value>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Define input directory and output file
    let json_dir = "/tmp/checkpoints_json";
    let output_file = "filtered_transactions.json";
    let package_id = "0x8d7866b423b15c3ae4c3b3737a4cd483b2ac720c3f1cf7dd67403f5a2dfa01d9";

    // Create output directory if it doesn't exist
    fs::create_dir_all(json_dir)?;

    // Collect filtered transactions
    let mut filtered_transactions = Vec::new();
    let dir = fs::read_dir(json_dir)?;
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
            // Read JSON file
            let mut file = File::open(&path)?;
            let mut contents = String::new();
            file.read_to_string(&mut contents)?;

            // Parse JSON
            let checkpoint: Value = serde_json::from_str(&contents)?;
            let sequence_number = checkpoint["checkpoint_summary"]["sequence_number"]
                .as_u64()
                .unwrap_or(0);

            // Filter transactions for package ID
            if let Some(transactions) = checkpoint["transactions"].as_array() {
                for tx in transactions {
                    let matches_package = tx["transaction"]["data"]
                        .as_array()
                        .map(|data_array| {
                            data_array.iter().any(|data| {
                                data["intent_message"]["value"]["V1"]["kind"]["ProgrammableTransaction"]["commands"]
                                    .as_array()
                                    .map(|commands| {
                                        commands.iter().any(|command| {
                                            command.get("MoveCall")
                                                .and_then(|move_call| move_call["package"].as_str())
                                                .map(|pkg| pkg == package_id)
                                                .unwrap_or(false)
                                        })
                                    })
                                    .unwrap_or(false)
                            })
                        })
                        .unwrap_or(false);

                    if matches_package {
                        let transaction = tx["transaction"].clone();
                        let effects = tx["effects"].clone();
                        let events = tx.get("events").cloned();
                        filtered_transactions.push(FilteredTransaction {
                            checkpoint_sequence: sequence_number,
                            transaction,
                            effects,
                            events,
                        });
                        println!(
                            "Found transaction in checkpoint {} for package ID {}",
                            sequence_number, package_id
                        );
                    }
                }
            }
        }
    }

    // Save filtered transactions to JSON file
    let json_output = serde_json::to_string_pretty(&filtered_transactions)?;
    let mut json_file_handle = AsyncFile::create(output_file).await?;
    json_file_handle.write_all(json_output.as_bytes()).await?;
    println!(
        "Saved {} filtered transactions to {}",
        filtered_transactions.len(),
        output_file
    );

    Ok(())
}