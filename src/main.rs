use anyhow::{Context, Result};
use bip39::{Language, Mnemonic};
use chrono::{DateTime, TimeZone, Utc};
use clearscreen;
use dialoguer::Select;
use reqwest::blocking::Client;
use serde_json::Value;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;

struct ZcashClient {
    url: String,
    username: String,
    password: String,
    http: Client,
}

impl ZcashClient {
    fn new(base_url: &str, port: u16) -> Result<Self> {
        let cookie_path = PathBuf::from("/var/lib/zebrad-rpc/.cookie");
        let (username, password) = if cookie_path.exists() {
            let content = fs::read_to_string(&cookie_path)
                .context("Failed to read Zebra .cookie file")?;
            if let Some(pos) = content.find(':') {
                (
                    "__cookie__".to_string(),
                    content[pos + 1..].trim().to_string(),
                )
            } else {
                ("".to_string(), "".to_string())
            }
        } else {
            ("".to_string(), "".to_string())
        };

        let full_url = format!("{}:{}", base_url, port);
        let http = Client::new();
        let instance = Self {
            url: full_url,
            username,
            password,
            http,
        };

        instance.call("getinfo", None)?;
        println!("Successfully connected to Zebra node\n");
        Ok(instance)
    }

    fn call(&self, method: &str, params: Option<Value>) -> Result<Value> {
        let params = params.unwrap_or(Value::Array(vec![]));
        let body = serde_json::json!({
            "jsonrpc": "1.0",
            "id": "rusty-zechub",
            "method": method,
            "params": params
        });

        let resp = self.http
            .post(&self.url)
            .basic_auth(&self.username, Some(&self.password))
            .json(&body)
            .send()
            .context(format!("RPC call to {} failed", method))?;

        let json: Value = resp.json().context("Failed to parse JSON-RPC response")?;

        if let Some(err) = json.get("error") {
            if !err.is_null() {
                anyhow::bail!("RPC error: {}", err);
            }
        }
        Ok(json["result"].clone())
    }
}

fn clear() {
    let _ = clearscreen::clear();
}

fn main() -> Result<()> {
    let client = ZcashClient::new("http://127.0.0.1", 8232)?;

    loop {
        clear();
        let options = [
            "Display Mnemonic",
            "Visualize Mempool",
            "Blockchain Detail",
            "Extract Supply Info",
            "Extract Supply Info at Block",
            "List Transactions of Block",
            "Transaction Detail",
            "Transaction Type",
            "Transaction Date",
            "Block Detail",
            "Block Date",
            "Peer Details",
            "Exit",
        ];

        let selection = Select::new()
            .with_prompt("Rusty-Zechub — Zebra RPC Explorer")
            .items(&options)
            .default(0)
            .interact()?;

        clear();

        match selection {
            0 => display_mnemonic()?,
            1 => visualize_mempool(&client)?,
            2 => { get_blockchain_info(&client, false)?; },
            3 => extract_supply_info(&client)?,
            4 => extract_supply_at_block(&client)?,
            5 => list_block_transactions(&client)?,
            6 => { transaction_detail(&client, false)?; },
            7 => transaction_type(&client)?,
            8 => transaction_date(&client)?,
            9 => block_detail(&client)?,
            10 => block_date(&client)?,
            11 => peer_details(&client)?,
            12 => {
                println!("Goodbye!");
                break;
            }
            _ => unreachable!(),
        }

        if selection != 12 {
            print!("\nPress Enter to return to menu...");
            io::stdout().flush()?;
            let mut dummy = String::new();
            let _ = io::stdin().read_line(&mut dummy);
        }
    }
    Ok(())
}

fn display_mnemonic() -> Result<()> {
    let mnemonic = Mnemonic::generate_in(Language::English, 24)
        .map_err(|e| anyhow::anyhow!("Failed to generate mnemonic: {}", e))?;
    println!("Generated 24-word BIP39 Mnemonic (English):\n");
    println!("{}", mnemonic);
    println!("\nSave this securely offline. Never share it!");
    Ok(())
}

fn visualize_mempool(client: &ZcashClient) -> Result<()> {
    let mempool = client.call("getrawmempool", Some(serde_json::json!([true])))?;
    println!("Mempool (verbose):\n");
    println!("{}", serde_json::to_string_pretty(&mempool)?);
    if let Some(tx_array) = mempool.as_array() {
        println!("\nTotal transactions in mempool: {}", tx_array.len());
    }
    Ok(())
}

fn get_blockchain_info(client: &ZcashClient, quiet: bool) -> Result<Value> {
    let info = client.call("getblockchaininfo", None)?;
    if !quiet {
        println!("Blockchain Info:\n");
        println!("{}", serde_json::to_string_pretty(&info)?);
    }
    Ok(info)
}

// ==================== UPDATED SUPPLY FUNCTIONS WITH NICE FORMATTING ====================

fn extract_supply_info(client: &ZcashClient) -> Result<()> {
    let info = get_blockchain_info(client, true)?;
    let height = info["blocks"].as_i64();
    print_supply_with_size(&info, height, &info);
    Ok(())
}

fn extract_supply_at_block(client: &ZcashClient) -> Result<()> {
    print!("Enter block height: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let block_str = input.trim();

    let block_data = client.call("getblock", Some(serde_json::json!([block_str, 1])))?;
    let chain_info = get_blockchain_info(client, true)?;

    print_supply_with_size(&block_data, block_data["height"].as_i64(), &chain_info);
    Ok(())
}

fn print_supply_with_size(supply_data: &Value, height: Option<i64>, size_data: &Value) {
    if let Some(h) = height {
        println!("At block: {}", h);
    }

    println!("-----------------------------------------------");

    // Size on disk - aligned inside the box
    if let Some(size_bytes) = size_data["size_on_disk"].as_u64() {
        let size_gb = size_bytes as f64 / (1024.0 * 1024.0 * 1024.0);
        println!("Chain size on disk : {:.4} GB", size_gb);
    } else if let Some(block_size) = supply_data["size"].as_u64() {
        let size_mb = block_size as f64 / (1024.0 * 1024.0);
        println!("This block size     : {:.2} MB", size_mb);
    } else {
        println!("Size on disk        : (not available)");
    }

    // Supply pools - all aligned at ":"
    let empty = vec![];
    let pools = supply_data["valuePools"].as_array().unwrap_or(&empty);

    let get_value = |pool: &Value| -> f64 {
        pool["chainValue"].as_f64()
            .or_else(|| pool["chainValueZat"].as_i64().map(|v| v as f64 / 100_000_000.0))
            .unwrap_or(0.0)
    };

    let transparent = if pools.len() > 0 { get_value(&pools[0]) } else { 0.0 };
    let sprout      = if pools.len() > 1 { get_value(&pools[1]) } else { 0.0 };
    let sapling     = if pools.len() > 2 { get_value(&pools[2]) } else { 0.0 };
    let orchard     = if pools.len() > 3 { get_value(&pools[3]) } else { 0.0 };
    let lockbox     = if pools.len() > 4 { get_value(&pools[4]) } else { 0.0 };

    println!("Transparent Pool    : {:.8} ZEC", transparent);
    println!("Sprout Pool         : {:.8} ZEC", sprout);
    println!("Sapling Pool        : {:.8} ZEC", sapling);
    println!("Orchard Pool        : {:.8} ZEC", orchard);
    println!("Lockbox             : {:.8} ZEC", lockbox);
    println!("-----------------------------------------------");
    println!("Total Shielded      : {:.8} ZEC\n", sapling + sprout + orchard);
}

// ==================== Remaining functions unchanged ====================

fn transaction_detail(client: &ZcashClient, quiet: bool) -> Result<Value> {
    print!("Enter txid: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let txid = input.trim();
    let tx = client.call("getrawtransaction", Some(serde_json::json!([txid, 1])))?;
    if !quiet {
        println!("\nTransaction Detail:\n");
        println!("{}", serde_json::to_string_pretty(&tx)?);
    }
    Ok(tx)
}

fn transaction_type(client: &ZcashClient) -> Result<()> {
    let tx = transaction_detail(client, true)?;
    let mut types = vec![];

    let is_coinbase = tx["vin"]
        .as_array()
        .and_then(|vin| vin.get(0))
        .and_then(|v| v["coinbase"].as_str())
        .is_some();

    if let Some(vin) = tx["vin"].as_array() {
        if !vin.is_empty() && !is_coinbase {
            types.push("Transparent Input");
        }
    }
    if let Some(vout) = tx["vout"].as_array() {
        if !vout.is_empty() {
            types.push("Transparent Output");
        }
    }
    if let Some(spends) = tx["vShieldedSpend"].as_array() {
        if !spends.is_empty() { types.push("Sapling Spend"); }
    }
    if let Some(outputs) = tx["vShieldedOutput"].as_array() {
        if !outputs.is_empty() { types.push("Sapling Output"); }
    }
    if let Some(orchard) = tx.get("orchard").and_then(|o| o.get("actions")) {
        if let Some(arr) = orchard.as_array() {
            if !arr.is_empty() { types.push("Orchard Action"); }
        }
    }

    println!("Transaction Type Analysis:");
    if is_coinbase {
        println!(" This is a COINBASE transaction");
    }
    if types.is_empty() && !is_coinbase {
        println!(" This appears to be a special / unknown transaction");
    } else {
        for t in types {
            println!(" {}", t);
        }
    }

    let mut transparent_total = 0.0;
    if let Some(vout) = tx["vout"].as_array() {
        for out in vout {
            if let Some(val) = out["value"].as_f64() {
                transparent_total += val;
            } else if let Some(zat) = out["valueZat"].as_i64() {
                transparent_total += zat as f64 / 100_000_000.0;
            }
        }
    }
    println!("\nTransparent value (outputs): {:.8} ZEC", transparent_total);

    if let Some(vb) = tx["valueBalance"].as_f64() {
        println!("Sapling valueBalance : {:.8} ZEC", vb);
    }
    if let Some(vb_zat) = tx["valueBalanceZat"].as_i64() {
        println!("Sapling valueBalanceZat : {} zats", vb_zat);
    }
    if let Some(orch) = tx.get("orchard") {
        if let Some(vb) = orch["valueBalance"].as_f64() {
            println!("Orchard valueBalance : {:.8} ZEC", vb);
        }
        if let Some(vb_zat) = orch["valueBalanceZat"].as_i64() {
            println!("Orchard valueBalanceZat : {} zats", vb_zat);
        }
    }

    if let Some(height) = tx["height"].as_i64() {
        println!("Mined in block : {}", height);
    }
    if let Some(time) = tx["time"].as_i64() {
        let dt: DateTime<Utc> = Utc.timestamp_opt(time, 0).unwrap();
        println!("Timestamp : {}", dt.to_rfc2822());
    }
    Ok(())
}

fn transaction_date(client: &ZcashClient) -> Result<()> {
    let tx = transaction_detail(client, true)?;
    let timestamp = tx["time"].as_i64().or_else(|| tx["blocktime"].as_i64());
    if let Some(ts) = timestamp {
        let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
        println!("Transaction timestamp : {}", dt.to_rfc2822());
        println!("Unix timestamp : {}", ts);
    } else {
        println!("No timestamp found in transaction data (unconfirmed?)");
    }
    Ok(())
}

fn block_detail(client: &ZcashClient) -> Result<()> {
    print!("Enter block height or hash: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let block_id = input.trim();
    let block = client.call("getblock", Some(serde_json::json!([block_id, 1])))?;
    println!("\nBlock Detail:\n");
    println!("{}", serde_json::to_string_pretty(&block)?);
    Ok(())
}

fn block_date(client: &ZcashClient) -> Result<()> {
    print!("Enter block height or hash: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let block_id = input.trim();
    let block = client.call("getblock", Some(serde_json::json!([block_id, 1])))?;
    if let Some(ts) = block["time"].as_i64() {
        let dt: DateTime<Utc> = Utc.timestamp_opt(ts, 0).unwrap();
        println!("Block timestamp : {}", dt.to_rfc2822());
        println!("Height : {}", block["height"].as_i64().unwrap_or(0));
    } else {
        println!("No timestamp found");
    }
    Ok(())
}

fn peer_details(client: &ZcashClient) -> Result<()> {
    let peers = client.call("getpeerinfo", None)?;
    let count = peers.as_array().map(|a| a.len()).unwrap_or(0);
    println!("Connected peers: {}", count);
    println!("{}", serde_json::to_string_pretty(&peers)?);
    Ok(())
}

fn list_block_transactions(client: &ZcashClient) -> Result<()> {
    print!("Enter block height or hash: ");
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    let block_id = input.trim();
    let block = client.call("getblock", Some(serde_json::json!([block_id, 1])))?;
    if let Some(txs) = block["tx"].as_array() {
        println!("Transactions in block ({} total):\n", txs.len());
        for (i, tx) in txs.iter().enumerate() {
            println!(" {:3}. {}", i + 1, tx.as_str().unwrap_or("???"));
        }
    }
    Ok(())
}
