# Rusty-ZecHub

A playground for RUST.

* Working with JSON's in RUST
* Output a random seed phrase
* Working with Zebrad
  * Extract ZEC supply using Zebrad
  * Exctract ZEC supply at any given block
  * Extract size of Zebra node on disk
  * Output current mempool with details
  * List tranactions of a given block
  * View transaction details given any txid
  * View transaction type (Transparent/Sapling/Orchard) given any txid
  * View transaction date given either a txid or block
  * View block details given any block
  * View peers connected to your node

`git clone https://github.com/dismad/rusty-zechub.git`

then

```bash
cd rusty-zechub
cargo run --release
```
***note***

* You will need a fully synced *zebrad* reachable via `http://127.0.0.1:8232`
* `cargo install --git https://github.com/ZcashFoundation/zebra zebrad`

* Edit zebrad.toml 
```
[rpc]
listen_addr = "127.0.0.1:8232"
enable_cookie_auth = true
```

* Start zebra:
  
  `zebrad start`

* uses `jq` for JSON formatting
  * `sudo apt install jq`

* `chmod +x getDateFromBlock.sh getDateFromTX.sh getTXfee.sh toCurl.sh tx_type.sh txDetails.sh`



![Screenshot_2025-06-24_22-17-48](https://github.com/user-attachments/assets/d42a4f69-c862-4db2-a8d7-04d4f417c6e0)


