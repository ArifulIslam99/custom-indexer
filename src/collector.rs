// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Import necessary crates and modules
use anyhow::Result; // For error handling, provides the Result type with anyhow errors
use async_trait::async_trait; // For using async functions in traits
use sui_data_ingestion_core::{setup_single_workflow, Worker}; // Import workflow setup and Worker trait
use sui_types::full_checkpoint_content::CheckpointData; // Import the CheckpointData type
use std::fs; // For synchronous filesystem operations (directory creation)
use std::path::Path; // For handling filesystem paths
use tokio::fs::File; // For asynchronous file operations
use tokio::io::AsyncWriteExt; // For async write operations on files

// Define a struct to represent our custom worker
struct CustomWorker {
    checkpoint_dir: String, // Directory where checkpoints will be saved
}

impl CustomWorker {
    /// Constructs a new CustomWorker.
    /// Ensures the checkpoint directory exists, creating it if necessary.
    ///
    /// # Arguments
    /// * `checkpoint_dir` - The directory path where checkpoints will be stored.
    fn new(checkpoint_dir: String) -> Self {
        // Attempt to create the directory (and any parent directories) if it doesn't exist.
        // If creation fails, print an error message to stderr.
        if let Err(e) = fs::create_dir_all(&checkpoint_dir) {
            eprintln!("Failed to create checkpoint directory: {}", e);
        }
        // Return the constructed CustomWorker instance.
        Self { checkpoint_dir }
    }
}

// Implement the Worker trait for CustomWorker, allowing it to be used in the workflow.
// The async_trait macro is required because Rust traits do not natively support async functions.
#[async_trait]
impl Worker for CustomWorker {
    // Define the associated result type for this worker.
    type Result = ();

    /// Asynchronously processes a single checkpoint.
    ///
    /// # Arguments
    /// * `checkpoint` - Reference to the checkpoint data to process.
    ///
    /// # Returns
    /// * `Result<()>` - Returns Ok(()) on success, or an error if something fails.
    async fn process_checkpoint(&self, checkpoint: &CheckpointData) -> Result<()> {
        // Extract the checkpoint number from the checkpoint summary.
        let checkpoint_number = checkpoint.checkpoint_summary.sequence_number;

        // Print a message indicating which checkpoint is being processed.
        println!("Processing checkpoint: {}", checkpoint_number);

        // Construct the filename for the checkpoint file, e.g., "162000000.chk"
        let filename = format!("{}.chk", checkpoint_number);

        // Build the full file path by joining the checkpoint directory and filename.
        let filepath = Path::new(&self.checkpoint_dir).join(filename);

        // Serialize the checkpoint data to bytes using BCS (Binary Canonical Serialization),
        // which is the standard serialization format for Sui.
        match bcs::to_bytes(checkpoint) {
            Ok(serialized_data) => {
                // Print the serialized data as a hex string for readability.
                // println!("{}", hex::encode(&serialized_data));

                // If serialization succeeds, attempt to create the file asynchronously.
                match File::create(&filepath).await {
                    Ok(mut file) => {
                        // If file creation succeeds, attempt to write the serialized data to the file.
                        if let Err(e) = file.write_all(&serialized_data).await {
                            // If writing fails, print an error message.
                            eprintln!("Failed to write checkpoint {}: {}", checkpoint_number, e);
                        } else {
                            // If writing succeeds, print a success message.
                            println!("Saved checkpoint {} to {:?}", checkpoint_number, filepath);
                        }
                    }
                    Err(e) => {
                        // If file creation fails, print an error message.
                        eprintln!("Failed to create file for checkpoint {}: {}", checkpoint_number, e);
                    }
                }
            }
            Err(e) => {
                // If serialization fails, print an error message.
                eprintln!("Failed to serialize checkpoint {}: {}", checkpoint_number, e);
            }
        }
        // Always return Ok(()) to indicate the function completed, even if errors were printed.
        Ok(())
    }
}

// The main entry point of the program.
// The #[tokio::main] macro sets up the async runtime so we can use async/await in main.
#[tokio::main]
async fn main() -> Result<()> {
    // Define the directory where checkpoint files will be stored.
    let checkpoint_dir = "/tmp/checkpoints".to_string();

    // Set up the workflow executor and termination sender.
    // - CustomWorker::new(checkpoint_dir): our worker that will process checkpoints.
    // - "https://checkpoints.mainnet.sui.io".to_string(): the URL to fetch checkpoints from.
    // - 162000000: the initial checkpoint number to start from.
    // - 5: the number of concurrent checkpoint processing tasks.
    // - None: no extra reader options.
    let (executor, _term_sender) = setup_single_workflow(
        CustomWorker::new(checkpoint_dir),
        "https://checkpoints.testnet.sui.io".to_string(),
        213411264, // initial checkpoint number
        5,         // concurrency
        None,      // extra reader options
    )
    .await?; // Await the setup and propagate any errors.

    // Await the executor to run the workflow until completion.
    executor.await?;

    // Return Ok(()) to indicate successful execution.
    Ok(())
}