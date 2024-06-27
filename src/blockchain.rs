use serde::{Serialize, Deserialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt::Write;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub from: String,
    pub to: String,
    pub amount: u64,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub index: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub nonce: u64,
    pub hash: String,
}

pub struct Blockchain {
    pub chain: Vec<Block>,
    pub mempool: Vec<Transaction>,
    pub accounts: HashMap<String, u64>,
}

impl Blockchain {
    pub fn new() -> Self {
        Blockchain {
            chain: vec![Blockchain::create_genesis_block()],
            mempool: Vec::new(),
            accounts: HashMap::new(),
        }
    }

    pub fn create_genesis_block() -> Block {
        Block {
            index: 0,
            previous_hash: String::from("0"),
            timestamp: 0,
            transactions: Vec::new(),
            nonce: 0,
            hash: String::from("genesis_hash"),
        }
    }

    pub fn create_account(&mut self) -> String {
        let random_bytes = rand::random::<[u8; 32]>();
        let hash = Sha256::digest(&random_bytes);
        let mut account_id = String::with_capacity(hash.len() * 2);
        for byte in hash {
            write!(&mut account_id, "{:02x}", byte).expect("Unable to write");
        }
        self.accounts.insert(account_id.clone(), 50); // Initial balance of 50
        account_id
    }

    pub fn add_transaction(&mut self, from: String, to: String, amount: u64) -> bool {
        if let Some(balance) = self.accounts.get_mut(&from) {
            if *balance >= amount {
                *balance -= amount;
                let transaction = Transaction { from: from.clone(), to: to.clone(), amount };
                self.mempool.push(transaction);
                if let Some(to_balance) = self.accounts.get_mut(&to) {
                    *to_balance += amount;
                } else {
                    self.accounts.insert(to.clone(), amount);
                }
                return true;
            }
        }
        false
    }

    pub fn mine_block(&mut self) -> Block {
        let last_block = self.chain.last().unwrap();
        let new_block = Block {
            index: last_block.index + 1,
            previous_hash: last_block.hash.clone(),
            timestamp: Blockchain::current_timestamp(),
            transactions: self.mempool.clone(),
            nonce: 0,
            hash: String::new(),
        };
        let mined_block = self.proof_of_work(new_block);
        self.chain.push(mined_block.clone());
        self.mempool.clear();
        mined_block
    }

    pub fn proof_of_work(&mut self, mut block: Block) -> Block {
        while !Blockchain::is_valid_hash(&block.hash) {
            block.nonce += 1;
            block.hash = Blockchain::hash(&block);
        }
        block
    }

    pub fn hash(block: &Block) -> String {
        let block_string = serde_json::to_string(block).expect("Failed to serialize block");
        let mut hasher = Sha256::new();
        hasher.update(block_string);
        let result = hasher.finalize();
        let mut hash_string = String::new();
        for byte in result {
            write!(&mut hash_string, "{:02x}", byte).expect("Unable to write");
        }
        hash_string
    }

    pub fn is_valid_hash(hash: &str) -> bool {
        hash.starts_with("0000")
    }

    pub fn current_timestamp() -> u64 {
        use std::time::{SystemTime, UNIX_EPOCH};
        let start = SystemTime::now();
        let since_epoch = start.duration_since(UNIX_EPOCH).expect("Time went backwards");
        since_epoch.as_secs()
    }

    pub fn add_block(&mut self, block: Block) -> bool {
        if self.validate_block(&block) {
            for tx in &block.transactions {
                if !self.apply_transaction(tx) {
                    return false;
                }
            }
            self.chain.push(block);
            return true;
        }
        false
    }

    pub fn validate_block(&self, block: &Block) -> bool {
        let last_block = self.chain.last().unwrap();
        if block.previous_hash != last_block.hash {
            return false;
        }
        if block.index != last_block.index + 1 {
            return false;
        }
        if block.hash != Blockchain::hash(&block) {
            return false;
        }
        true
    }

    fn apply_transaction(&mut self, tx: &Transaction) -> bool {
        if let Some(balance) = self.accounts.get_mut(&tx.from) {
            if *balance >= tx.amount {
                *balance -= tx.amount;
                *self.accounts.entry(tx.to.clone()).or_insert(0) += tx.amount;
                return true;
            }
        }
        false
    }

    pub fn get_balance(&self, account: &str) -> u64 {
        *self.accounts.get(account).unwrap_or(&0)
    }
}
