use solana_program::{
    account_info::AccountInfo, entrypoint, entrypoint::ProgramResult, program::invoke,
    pubkey::Pubkey,
};

entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    match spl_token::instruction::TokenInstruction::unpack(instruction_data).unwrap() {
        spl_token::instruction::TokenInstruction::TransferChecked { amount, .. } => {
            let source = &accounts[0];
            let mint = &accounts[1];
            let destination = &accounts[2];
            let authority = &accounts[3];
            invoke(
                &spl_token::instruction::transfer(
                    mint.key,
                    destination.key,
                    source.key,
                    authority.key,
                    &[],
                    amount,
                )
                .unwrap(),
                &[
                    source.clone(),
                    mint.clone(),
                    destination.clone(),
                    authority.clone(),
                ],
            )
        }
        _ => {
            panic!("wrong ix")
        }
    }
}