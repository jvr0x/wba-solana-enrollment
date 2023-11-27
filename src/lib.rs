mod programs;

#[cfg(test)]
mod tests {
    use super::*;
    use solana_client::rpc_client::RpcClient;
    use solana_program::message::Message;
    use solana_program::system_instruction::transfer;
    use solana_program::system_program;
    use solana_sdk::{
        bs58,
        pubkey::Pubkey,
        signature::{Keypair, Signer},
    };
    use std::io;
    use std::io::BufRead;
    use std::str::FromStr;

    use solana_sdk::signature::read_keypair_file;
    use solana_sdk::transaction::Transaction;

    use crate::programs::wba_prereq::{CompleteArgs, UpdateArgs, WbaPrereqProgram};

    const RPC_URL: &str = "https://api.devnet.solana.com";

    #[test]
    fn keygen() {
        let kp = Keypair::new();
        println!("pubkey: {}", kp.pubkey().to_string());
        println!("pubkey bytes : {:?}", kp.to_bytes());
    }

    #[test]
    fn airdrop() {
        // let keypair = Keypair::from_bytes();
        let keypair =
            read_keypair_file("./src/solana_dev_wallet2.json").expect("Couldn't find wallet file");
        let client = RpcClient::new(RPC_URL);

        match client.request_airdrop(&keypair.pubkey(), 2_000_000_000u64) {
            Ok(s) => {
                println!("Success! Check out your TX here:");
                println!(
                    "https://explorer.solana.com/tx/{}?cluster=devnet",
                    s.to_string()
                );
            }
            Err(e) => println!("Oops, something went wrong: {}", e.to_string()),
        };
    }

    #[test]
    fn transfer_sol() {
        let keypair =
            read_keypair_file("./src/solana_dev_wallet2.json").expect("Couldn't find wallet file");
        // Define our WBA public key
        let to_pubkey = Pubkey::from_str("7qRhLkkRq1whkZtDkWsoMCF6Hjr9CByytVYNHDm9aHfS").unwrap();
        let client = RpcClient::new(RPC_URL);

        // In order to sign transactions, we're going to need to get a recent blockhash, as signatures are.
        // designed to expire as a security feature:

        // Get recent blockhash
        let recent_blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, 1_000_000)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:
https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );

        let balance = client
            .get_balance(&keypair.pubkey())
            .expect("Failed to get balance");

        // Create a test transaction to calculate fees
        let message = Message::new_with_blockhash(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance)],
            Some(&keypair.pubkey()),
            &recent_blockhash,
        );

        // Calculate exact fee rate to transfer entire SOL amount out of account minus fees
        let fee = client
            .get_fee_for_message(&message)
            .expect("Failed to get fee calculator");

        // Deduct fee from lamports amount and create a TX with correct balance
        let transaction = Transaction::new_signed_with_payer(
            &[transfer(&keypair.pubkey(), &to_pubkey, balance - fee)],
            Some(&keypair.pubkey()),
            &vec![&keypair],
            recent_blockhash,
        );

        // Send the transaction
        let signature = client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        println!(
            "Success! Check out your TX here:
https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }

    #[test]
    fn base58_to_wallet() {
        println!("Input your private key as base58:");
        let stdin = io::stdin();
        let base58 = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your wallet file is:");
        let wallet = bs58::decode(base58)
            .into_vec()
            .unwrap();
        println!("{:?}", wallet);
    }

    #[test]
    fn wallet_to_base58() {
        println!("Input your private key as a wallet file byte array:");
        let stdin = io::stdin();
        let wallet = stdin.lock().lines().next().unwrap().unwrap();
        println!("Your private key is:");
        let base58 = bs58::encode(wallet).into_string();
        println!("{:?}", base58);
    }

    #[test]
    fn x() {
        let args = CompleteArgs {
            github: b"jvr0x".to_vec(),
        };
        let client = RpcClient::new(RPC_URL);

        let blockhash = client
            .get_latest_blockhash()
            .expect("Failed to get recent blockhash");

        let signer = read_keypair_file("./src/wba_dev_wallet.json").expect("Couldn't find wallet file");

        // PDA
        let prereq = WbaPrereqProgram::derive_program_address(&[
            b"prereq",
            signer.pubkey().to_bytes().as_ref(),
        ]);

        // Now we can invoke the "complete" function
        let transaction = WbaPrereqProgram::complete(
            &[&signer.pubkey(), &prereq, &system_program::id()],
            &args,
            Some(&signer.pubkey()),
            &[&signer],
            blockhash,
        );

        let signature = client
            .send_and_confirm_transaction(&transaction)
            .expect("Failed to send transaction");

        // Print our transaction out
        println!(
            "Success! Check out your TX here: https://explorer.solana.com/tx/{}/?cluster=devnet",
            signature
        );
    }
}
