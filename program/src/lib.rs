use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
    rent::Rent,
    sysvar::Sysvar,
};

// Every solana program has one entry point
// And it is convention to name it `process_instruction`.
// It should take in program_id, accounts, instruction_data as parameter.
fn process_instruction(
    // program id is noting but the id of this program(smart contract) on the solana network 
    program_id: &Pubkey,
    // array of account that is going to be used to process instruction
    // As you can see it is a array of AccountInfo.
    // We can provide as many as we want.
    accounts: &[AccountInfo],
    // This is the data we want to process our instuction for.
    // It is a list of 8 bitunsinged integers(0..255).
    instruction_data: &[u8],
) -> ProgramResult {

    // We check if we have a instruction_data len greater then 0, if it is not, we do not want to procced.
    // So we return Error with InvalidInstructionData Message.
    if instruction_data.len() == 0 {
        return Err(ProgramError::InvalidInstructionData);
    }

    // As we know that solana program have only one entrypoint, but we want three entry point for your program.
    // In order to fix this, we are going to take advantage of the fact that there is no limit to the instruction_data array 😜
    // we use the first element of the instruction_data array to know what entry point we want to call.
    // Now we just check and call the funciton for each of them.
    // 0 for create_campaign,
    // 1 for withdraw
    // 2 for donate.
    if instruction_data[0] == 0 {
        return create_campaign(
            program_id,
            accounts,
            // Notice we pass program_id and accounts as they were,
            // but we pass a reference to silce of [instruction_data].
            // we do not want the first element in any of our function
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] == 1 {
        return withdraw(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    } else if instruction_data[0] ==2 {
        return donate(
            program_id,
            accounts,
            &instruction_data[1..instruction_data.len()],
        );
    }

    // If instruction data doesn't match we give an error.
    msg!("Didn't find the required entrypoint ");
    Err(ProgramError::InvalidInstructionData)
}

// Then we call the entry point macro to add `process_instruction` as our entrypoint to our program
entrypoint!(process_instruction);

// Here I have created the function for every action we want to do in our program.
// They take same parameter as in process_instruction and same return type
#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct CampaignDetails {
    pub admin: Pubkey,
    pub name: String,
    pub description: String,
    pub image_link: String,
    pub amount_donated: u64,
}
fn create_campaign(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {

    // We create a iterator an accounts
    // account parameter is the array of accounts related to this entrypoint
    let accounts_iter = &mut accounts.iter();

    // writing accounts or we can call it program account
    // This is an account we will create in our front-end.
    // This account should be owned by the solana program
    let writing_account = next_account_info(accounts_iter)?;
    
    // Accounts of the person creating the campaign, signer
    let creator_account = next_account_info(accounts_iter)?;

    // Now to allow transcation we want the creator account to sign the transcation.
    if !creator_account.is_signer {
        msg!("creator_account should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    // We want to write in this account so we want it is owned by the program.
    if writing_account.owner != program_id {
        msg!("writing_accounts isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }

    // By deriving the trait BorshDeserializer in our CampaignDetails struct we have added a method `try_from_slice` which take in the parameter array of u8 and create
    // an object of CampaignDetails with it.
    let mut input_data = CampaignDetails::try_from_slice(&instruction_data)
        .expect("Instruction data serialization didn't work");

    // Validating that only admin can create campaign
    if input_data.admin != *creator_account.key {
        msg!("Invalid instruction data");
        return Err(ProgramError::InvalidInstructionData);
    }

    // let try to make our program rent exempet
    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());
    if **writing_account.lamports.borrow() < rent_exemption {
        msg!("The balance of writing_account should be more then rent_exemption");
        return Err(ProgramError::InsufficientFunds);
    }

    // Then we can set the initial amount donated to be zero.
    input_data.amount_donated = 0;

    // writing into CampaignDetails
    input_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;

    Ok(())
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct WithdrawRequest {
    pub amount: u64,
}
fn withdraw(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let admin_account = next_account_info(accounts_iter)?;

    // we check if writing program is owned by program
    if writing_account.owner != program_id {
        msg!("writing account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    // Admin accounts should be the signer in this transaction
    if !admin_account.is_signer {
        msg!("Admin should be signer");
        return Err(ProgramError::IncorrectProgramId)
    }
    let campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow())
        .expect("Error deserializing data");

    // Then we check if the admin_account's public key is equal to 
    // the public key we have stored in our campaing_data.
    if campaign_data.admin != *admin_account.key {
        msg!("Only the account admin can withdraw");
        return Err(ProgramError::InvalidAccountData);
    }

    let input_data = WithdrawRequest::try_from_slice(&instruction_data)
        .expect("Instruction data serialization didn't worked");

    let rent_exemption = Rent::get()?.minimum_balance(writing_account.data_len());

    // we check if we have enough funds
    if **writing_account.lamports.borrow() - rent_exemption < input_data.amount {
        msg!("Insufficent balance");
        return Err(ProgramError::InsufficientFunds);
    }

    // Transfer balance
    // we will decrease the balance of the program account, and increase the admin_account balance.
    **writing_account.try_borrow_mut_lamports()? -= input_data.amount; //  we can only decrease the balance of a program-owned account.
    **admin_account.try_borrow_mut_lamports()? += input_data.amount;
    Ok(())
}

fn donate(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();
    let writing_account = next_account_info(accounts_iter)?;
    let donator_program_account = next_account_info(accounts_iter)?;
    let donator = next_account_info(accounts_iter)?;

    if writing_account.owner != program_id {
        msg!("writing_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    if donator_program_account.owner != program_id {
        msg!("donator_program_account isn't owned by program");
        return Err(ProgramError::IncorrectProgramId);
    }
    if !donator.is_signer {
        msg!("donator should be signer");
        return Err(ProgramError::IncorrectProgramId);
    }

    let mut campaign_data = CampaignDetails::try_from_slice(*writing_account.data.borrow())
        .expect("Error deserializing data");

    campaign_data.amount_donated += **donator_program_account.lamports.borrow();

    **writing_account.try_borrow_mut_lamports()? += **donator_program_account.lamports.borrow();
    **donator_program_account.try_borrow_mut_lamports()? = 0;

    campaign_data.serialize(&mut &mut writing_account.data.borrow_mut()[..])?;
    Ok(())
}
