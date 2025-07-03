// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use serde_json::Value;
use std::fs::File;
use std::io::Read;
use tokio_postgres::NoTls;
use tokio::task;

#[tokio::main]
async fn main() -> Result<()> {
    // Read the JSON file
    let json_file = "filtered_transactions.json";
    let mut file = File::open(json_file)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let transactions: Vec<Value> = serde_json::from_str(&contents)?;
    println!("Found {} transactions", transactions.len());

    // Connect to PostgreSQL
    let (client, connection) = tokio_postgres::connect(
        "host=localhost user=alice password=123 dbname=checkpoint_data",
        NoTls,
    )
    .await?;
    println!("Connected to database");

    // Spawn connection task
    task::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("Connection error: {}", e);
        }
    });

    // Process transactions
    for tx in transactions {
        // Extract inputs array for data
        let data = &tx["transaction"]["data"][0]["intent_message"]["value"]["V1"]["kind"]["ProgrammableTransaction"]["inputs"];
        if !data.is_array() {
            return Err(anyhow::anyhow!("Invalid inputs array"));
        }
        println!("Checkpoint: {} inputs", data.as_array().unwrap().len());

        // Extract function name from MoveCall
        let commands = tx["transaction"]["data"][0]["intent_message"]["value"]["V1"]["kind"]["ProgrammableTransaction"]["commands"]
            .as_array()
            .ok_or_else(|| anyhow::anyhow!("Invalid commands array"))?;
        let function_name = commands
            .get(0)
            .and_then(|cmd| cmd.get("MoveCall"))
            .and_then(|move_call| move_call.get("function"))
            .and_then(|func| func.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid function name"))?
            .to_string();
        println!("Function name: {}", function_name);

        // Extract sender
        let tx_sender = tx["transaction"]["data"][0]["intent_message"]["value"]["V1"]["sender"]
            .as_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid sender"))?
            .to_string();

        // Extract gas_used object
        let gas_cost = &tx["effects"]["V2"]["gas_used"];
        if !gas_cost.is_object() {
            return Err(anyhow::anyhow!("Invalid gas_used object"));
        }
        println!("Gas cost object: {:?}", gas_cost);

        // Insert into PostgreSQL
        let rows_affected = client
            .execute(
                "INSERT INTO nft_transactions (tx_sender, function_name, data, gas_cost) VALUES ($1, $2, $3, $4)",
                &[&tx_sender, &function_name, &data, &gas_cost],
            )
            .await?;
        println!(
            "Inserted transaction: sender={}, function_name={}, data_len={}, gas_cost={:?}, rows_affected={}",
            tx_sender, function_name, data.as_array().unwrap().len(), gas_cost, rows_affected
        );
    }

    Ok(())
}