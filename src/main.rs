mod node;
mod blockchain;

use node::Node;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

fn main() {
    let node1 = Arc::new(Node::new("127.0.0.1:3030"));
    let node2 = Arc::new(Node::new("127.0.0.1:3031"));

    let node1_clone = node1.clone();
    let node2_clone = node2.clone();

    thread::spawn(move || {
        node1_clone.start().unwrap();
    });

    thread::spawn(move || {
        node2_clone.start().unwrap();
    });

    thread::sleep(Duration::from_secs(5));
    node1.add_peer("127.0.0.1:3031".to_string());
    node2.add_peer("127.0.0.1:3030".to_string());

    thread::sleep(Duration::from_secs(5));

    // Create accounts
    let account1 = node1.blockchain.lock().unwrap().create_account();
    let account2 = node2.blockchain.lock().unwrap().create_account();

    // Check balances
    let balance1 = node1.blockchain.lock().unwrap().get_balance(&account1);
    let balance2 = node2.blockchain.lock().unwrap().get_balance(&account2);

    println!("Balance1: {}, Balance2: {}", balance1, balance2);

    println!("Account1: {}, Account2: {}", account1, account2);

    // Send transaction from account1 to account2
    let success = node1.blockchain.lock().unwrap().add_transaction(account1.clone(), account2.clone(), 23);
    println!("Transaction success: {}", success);

    // Mine a block
    node1.mine();

    thread::sleep(Duration::from_secs(5));

    // Check balances
    let balance3 = node1.blockchain.lock().unwrap().get_balance(&account1);
    let balance4 = node2.blockchain.lock().unwrap().get_balance(&account2);

    println!("Balance1: {}, Balance2: {}", balance3, balance4);
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    #[test]
    fn test_create_account() {
        let node = Arc::new(Node::new("127.0.0.1:3030"));
        node.start().unwrap();

        let account_id = node.blockchain.lock().unwrap().create_account();
        assert_eq!(account_id.len(), 64);
    }

    #[test]
    fn test_send_transaction() {
        let node = Arc::new(Node::new("127.0.0.1:3031"));
        node.start().unwrap();

        let account1 = node.blockchain.lock().unwrap().create_account();
        let account2 = node.blockchain.lock().unwrap().create_account();

        node.blockchain.lock().unwrap().add_transaction(account1.clone(), account2.clone(), 50);

        // Mine the block containing the transaction
        node.mine();

        let balance1 = node.blockchain.lock().unwrap().get_balance(&account1);
        let balance2 = node.blockchain.lock().unwrap().get_balance(&account2);

        assert_eq!(balance1, 50);
        assert_eq!(balance2, 100);
    }

    #[test]
    fn test_mine_block() {
        let node = Arc::new(Node::new("127.0.0.1:3032"));
        node.start().unwrap();

        let account1 = node.blockchain.lock().unwrap().create_account();
        let account2 = node.blockchain.lock().unwrap().create_account();

        node.blockchain.lock().unwrap().add_transaction(account1.clone(), account2.clone(), 50);

        // Mine the block containing the transaction
        node.mine();

        let balance1 = node.blockchain.lock().unwrap().get_balance(&account1);
        let balance2 = node.blockchain.lock().unwrap().get_balance(&account2);

        assert_eq!(balance1, 50);
        assert_eq!(balance2, 100);
    }

    #[test]
    fn test_blockchain_sync() {
        let node1 = Arc::new(Node::new("127.0.0.1:3035"));
        let node2 = Arc::new(Node::new("127.0.0.1:3036"));

        let node1_clone = node1.clone();
        let node2_clone = node2.clone();

        thread::spawn(move || {
            node1_clone.start().unwrap();
        });

        thread::spawn(move || {
            node2_clone.start().unwrap();
        });

        // Give nodes some time to start
        thread::sleep(Duration::from_secs(2));

        node1.add_peer("127.0.0.1:3036".to_string());
        node2.add_peer("127.0.0.1:3035".to_string());

        // Give peers some time to connect
        thread::sleep(Duration::from_secs(2));

        let account1 = node1.blockchain.lock().unwrap().create_account();
        let account2 = node2.blockchain.lock().unwrap().create_account();

        node1.blockchain.lock().unwrap().add_transaction(account1.clone(), account2.clone(), 50);

        // Mine the block containing the transaction
        node1.mine();

        // Give nodes some time to sync
        thread::sleep(Duration::from_secs(2));

        let node1_chain = node1.blockchain.lock().unwrap().chain.clone();
        let node2_chain = node2.blockchain.lock().unwrap().chain.clone();

        assert_eq!(node1_chain, node2_chain);
    }
}
