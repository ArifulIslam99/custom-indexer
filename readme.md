# Custom Indexer for WorldChannels (Testnet)

This Rust project processes Sui blockchain checkpoint data by downloading checkpoints, converting them to JSON, filtering transactions by package ID, and storing relevant data in a PostgreSQL database. It consists of four Rust binaries: `collector.rs`, `reader.rs`, `json_reader.rs`, and `store_db.rs`.

## Rust Files and Their Purpose

### collector.rs
- **Purpose**: Downloads checkpoint data from the Sui testnet and saves it as binary `.chk` files in `/tmp/checkpoints`.
- **Details**: Uses `sui_data_ingestion_core` to fetch checkpoints starting from a specified sequence number (e.g., 213411264) and serializes them using BCS.

### reader.rs
- **Purpose**: Converts binary `.chk` files to JSON format.
- **Details**: Reads `.chk` files from `/tmp/checkpoints`, deserializes them, and saves JSON files to `/tmp/checkpoints_json` (e.g., `213411264.json`).

### json_reader.rs
- **Purpose**: Filters JSON checkpoint files for transactions matching a specific package ID and saves them to `filtered_transactions.json`.
- **Details**: Filters transactions with `MoveCall` commands matching the package ID `0x8d7866b423b15c3ae4c3b3737a4cd483b2ac720c3f1cf7dd67403f5a2dfa01d9`.

### store_db.rs
- **Purpose**: Stores filtered transaction data in a PostgreSQL database.
- **Details**: Reads `filtered_transactions.json` and inserts `tx_sender`, `function_name`, `data` (full `inputs` array as JSONB), and `gas_cost` (`gas_used` object as JSONB) into the `nft_transactions` table.

## How to Execute the Project

### Prerequisites
- **Rust**: Install via `rustup` (https://rustup.rs/).
- **PostgreSQL**: Running with a database `checkpoint_data` and user `alice` (password: `123`).

### Setup
1. **Clone the Repository**:
   ```bash
   git clone https://github.com/ArifulIslam99/custom-checkpoint.git
   cd custom-checkpoint
   ```

2. **Set Up PostgreSQL**:
   ```bash
   sudo -u postgres psql
   ```
   ```sql
   CREATE DATABASE checkpoint_data;
   CREATE USER alice WITH PASSWORD '123';
   GRANT ALL PRIVILEGES ON DATABASE checkpoint_data TO alice;
   \c checkpoint_data
   GRANT ALL ON SCHEMA public TO alice;
   ALTER SCHEMA public OWNER TO alice;
   CREATE TABLE nft_transactions (
       id SERIAL PRIMARY KEY,
       tx_sender VARCHAR(66) NOT NULL,
       function_name TEXT NOT NULL,
       data JSONB NOT NULL,
       gas_cost JSONB NOT NULL
   );
   ```

3. **Build the Project**:
   ```bash
   cargo build --release
   ```

### Execution Flow
1. **Download Checkpoints**:
   ```bash
   cargo run --release --bin checkpoint-downloader
   ```

2. **Convert to JSON**:
   ```bash
   cargo run --release --bin checkpoint-reader
   ```

3. **Filter Transactions**:
   ```bash
   cargo run --release --bin json-reader
   ```

4. **Store in Database**:
   ```bash
   cargo run --release --bin store-db
   ```

5. **Verify Database**:
   ```bash
   psql -U alice -d checkpoint_data -h localhost
   ```
   ```sql
   SELECT * FROM nft_transactions;
   ```
