use crate::{
    error::Funds4GoodError, instruction::Funds4GoodInstruction, state, state::AccTypes,
    state::BorrowerAccount, state::GuarantorAccount, state::LenderAccountData,
    state::LoanInfoAccDataHeader, state::LoanInfoAccLendersData,
};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program::invoke_signed,
    program_error::ProgramError,
    program_pack::{IsInitialized, Pack},
    pubkey::Pubkey,
    sysvar::{clock::Clock, rent::Rent, Sysvar},
};

use spl_token::state::Account as TokenAccount;
use std::convert::TryInto;

const Funds4Good_COIN_DECIMALS: u64 = 1000_000_000;
const MIN_LENDING_AMOUNT: u64 = 10_000_000_000u64;
pub struct Processor;
impl Processor {
    pub fn process(
        program_id: &Pubkey,
        accounts: &[AccountInfo],
        instruction_data: &[u8],
    ) -> ProgramResult {
        msg!("in processor");
        let instruction = Funds4GoodInstruction::unpack(instruction_data)?;
        msg!("out instruction");
        match instruction {
            Funds4GoodInstruction::LendToBorrower {
                amount_to_lend_input,
                lender_id_input,
            } => {
                msg!("Funds4GoodInstruction::LendToBorrower");
                Self::process_lend_to_borrower(
                    accounts,
                    amount_to_lend_input,
                    lender_id_input,
                    program_id,
                )
            }
            Funds4GoodInstruction::WithdrawLenderFreeWalletFunds { lender_id_input } => {
                msg!("Funds4GoodInstruction::WithdrawLenderFreeWalletFunds");
                Self::process_withdraw_lender_free_wallet_funds(
                    accounts,
                    lender_id_input,
                    program_id,
                )
            }
            Funds4GoodInstruction::WithdrawCollectedLoanFunds {} => {
                msg!("Funds4GoodInstruction::WithdrawCollectedLoanFunds");
                Self::process_withdraw_collected_loan_funds(accounts, program_id)
            }

            Funds4GoodInstruction::TransferFunds4GoodVaultAccountOwnership {} => {
                msg!("Funds4GoodInstruction::TransferFunds4GoodVaultAccountOwnership");
                Self::process_transfer_Funds4Good_vault_account_ownership(accounts, program_id)
            }

            Funds4GoodInstruction::InitializeLendersStorageAccount {} => {
                msg!("Funds4GoodInstruction::InitializeLendersStorageAccount");
                Self::process_initialize_lenders_storage_account(accounts)
            }
            Funds4GoodInstruction::InitializeBorrowerAccount {} => {
                msg!("Funds4GoodInstruction::InitializeBorrowerAccount");
                Self::process_initialize_borrower_storage_account(accounts, program_id)
            }
            Funds4GoodInstruction::InitializeGuarantorAccount {} => {
                msg!("Funds4GoodInstruction::InitializeGuarantorAccount");
                Self::process_initialize_guarantor_storage_account(accounts, program_id)
            }

            Funds4GoodInstruction::PayEMIforLoan {
                emi_amount_to_pay_input,
            } => {
                msg!("Funds4GoodInstruction::PayEMIforLoan");
                Self::process_pay_emi(accounts, emi_amount_to_pay_input, program_id)
            }

            Funds4GoodInstruction::InitializeLoanInfoAccount {
                num_days_left_for_first_repayment_input,
                num_emis_needed_to_repay_the_loan_input,
                num_days_for_fundraising_input,
                total_loan_amount_input,
            } => {
                msg!("Funds4GoodInstruction::InitializeLoanInfoAccount");
                Self::initialize_loan_info_account(
                    accounts,
                    num_days_left_for_first_repayment_input,
                    num_emis_needed_to_repay_the_loan_input,
                    num_days_for_fundraising_input,
                    total_loan_amount_input,
                    program_id,
                )
            }

            Funds4GoodInstruction::AirdropUsersWithFunds4GoodTestCoins {} => {
                msg!("Funds4GoodInstruction::AirdropUsersWithFunds4GoodTestCoins");
                Self::process_airdrop_users_with_Funds4Good_test_coins(accounts, program_id)
            }

            Funds4GoodInstruction::TransferAirdropVaultAccountOwnership {} => {
                msg!("Funds4GoodInstruction::TransferAirdropVaultAccountOwnership");
                Self::process_transfer_airdrop_vault_account_ownership(accounts, program_id)
            }

            Funds4GoodInstruction::ReturnFundsToLenders { num_accounts_input } => {
                msg!("Funds4GoodInstruction::ReturnFundsToLenders");
                Self::process_return_funds_to_lenders(accounts, num_accounts_input, program_id)
            }

            Funds4GoodInstruction::CloseLoanInfoAccount {} => {
                msg!("Funds4GoodInstruction::CloseLoanInfoAccount");
                Self::process_close_loan_info_account(accounts)
            }
        }
    }

    fn process_lend_to_borrower(
        accounts: &[AccountInfo],
        amount_to_lend_input: u64,
        lender_id_input: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lender_main_account = next_account_info(account_info_iter)?;

        if !lender_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lender_Funds4Good_coin_account_to_debit = next_account_info(account_info_iter)?;
        let Funds4Good_coin_vault_account = next_account_info(account_info_iter)?;

        

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let Funds4Good_coin_vault_account_data_before =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_before = Funds4Good_coin_vault_account_data_before.amount;

        let (pda_Funds4Good_vault, _bump_seed) =
            Pubkey::find_program_address(&[b"Funds4GoodFinance"], program_id);

        if Funds4Good_coin_vault_account_data_before.owner != pda_Funds4Good_vault {
            return Err(Funds4GoodError::Funds4GoodVaultAccountDoesNotMatched.into());
        }

        let transfer_lending_amount_to_vault_ix = spl_token::instruction::transfer(
            token_program.key,
            lender_Funds4Good_coin_account_to_debit.key,
            Funds4Good_coin_vault_account.key,
            lender_main_account.key,
            &[],
            amount_to_lend_input,
        )?;

        msg!("Calling the token program to transfer lending amount to vault...");
        msg!(
            "amount of Funds4Good coin tokens to transfer {}, lender debit key {}",
            (amount_to_lend_input as f64 / Funds4Good_COIN_DECIMALS as f64),
            lender_Funds4Good_coin_account_to_debit.key.to_string()
        );

        invoke(
            &transfer_lending_amount_to_vault_ix,
            &[
                lender_Funds4Good_coin_account_to_debit.clone(),
                Funds4Good_coin_vault_account.clone(),
                lender_main_account.clone(),
                token_program.clone(),
            ],
        )?;

        let Funds4Good_coin_vault_account_data_after =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_after = Funds4Good_coin_vault_account_data_after.amount;
        msg!(
            "Funds4GoodCoin vault balance after: {}",
            Funds4Good_coin_vault_balance_after
        );
        let vault_balance_increased = Funds4Good_coin_vault_balance_after
            .checked_sub(Funds4Good_coin_vault_balance_before)
            .unwrap();

        if vault_balance_increased < MIN_LENDING_AMOUNT {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }
       
        let loan_info_storage_account = next_account_info(account_info_iter)?;
       
        if loan_info_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }
       
        let lenders_data_storage_account = next_account_info(account_info_iter)?;
    
        if lenders_data_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }
       
        let mut lenders_storage_data_byte_array =
            lenders_data_storage_account.try_borrow_mut_data()?;
        
        if lenders_storage_data_byte_array[0] != AccTypes::LendersAcc as u8 {
            return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
        }
      
        if lenders_storage_data_byte_array[1] != 1u8 {
            return Err(Funds4GoodError::ExpectedLendersAccNumNotMatched.into());
        }   
     
        if lender_id_input > 49_999u32 {
            return Err(Funds4GoodError::InvalidLenderIdInput.into());
        }
      
        let lender_si_in_lenders_data_byte_array: usize = 2usize
            + (lender_id_input as usize)
                .checked_mul(state::LENDER_ACC_DATA_SIZE)
                .unwrap();
        let lender_ei_in_lenders_data_byte_array: usize =
            lender_si_in_lenders_data_byte_array + state::LENDER_ACC_DATA_SIZE;
        let mut lender_acc_data: LenderAccountData = state::unpack_to_lender_account_data(
            &lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();
      
        if lender_acc_data.is_account_active != 1u8 {
            lender_acc_data.is_account_active = 1u8;
            lender_acc_data.lender_main_acc_pubkey = *lender_main_account.key;
        } else {
         
            if lender_acc_data.lender_main_acc_pubkey != *lender_main_account.key {
                return Err(Funds4GoodError::InvalidLenderIdInput.into());
            }
        }
       
        lender_acc_data.total_lending_amount = lender_acc_data
            .total_lending_amount
            .checked_add(vault_balance_increased as u128)
            .unwrap();
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_add(vault_balance_increased)
            .unwrap();
         
        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
        }

        let now = Clock::get()?.unix_timestamp as u64;

        if loan_info_header_data.fundraising_period_ending_timestamp < now
            && loan_info_header_data.total_amount_lended < loan_info_header_data.total_loan_amount
        {
            return Err(Funds4GoodError::FundraisingPeriodExpired.into());
        }
     
        if loan_info_header_data.total_amount_lended >= loan_info_header_data.total_loan_amount {
            return Err(Funds4GoodError::BorrowerAlreadyFunded.into());
        }

        let loan_info_lender_data_si: usize = state::LOAN_INFO_HEADER_DATA_BYTES
            + (loan_info_header_data.next_index_to_store_lender_data as usize)
                * state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data_ei: usize =
            loan_info_lender_data_si + state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let mut loan_info_lender_data: LoanInfoAccLendersData =
            state::unpack_to_loan_info_acc_lender_data(
                &loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
            )
            .unwrap();

        loan_info_lender_data.lender_main_acc_pubkey = *lender_main_account.key;
        loan_info_lender_data.lenders_data_storage_acc_number = 1u8;
        loan_info_lender_data.lender_id = lender_id_input;
        loan_info_lender_data.lent_amount = vault_balance_increased;

        state::pack_to_loan_info_acc_lender_data(
            loan_info_lender_data,
            &mut loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
        )
        .unwrap();

        loan_info_header_data.next_index_to_store_lender_data =
            loan_info_header_data.next_index_to_store_lender_data + 1;
        loan_info_header_data.total_amount_lended = loan_info_header_data
            .total_amount_lended
            .checked_add(vault_balance_increased)
            .unwrap();
        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    fn process_pay_emi(
        accounts: &[AccountInfo],
        emi_amount_to_pay_input: u64,

        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let borrower_main_account = next_account_info(account_info_iter)?;

        if !borrower_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let borrower_Funds4Good_coin_account_to_debit = next_account_info(account_info_iter)?;

        // Funds4Good_coin_vault_account is program controlled vault account and can be only controlled by our deployed program for debit funds or any kind of operation
        let Funds4Good_coin_vault_account = next_account_info(account_info_iter)?;
        


        let borrower_storage_account = next_account_info(account_info_iter)?;
        if borrower_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }

        /*
        let expected_borrower_storage_account_pubkey = Pubkey::create_with_seed(
            borrower_main_account.key,
            "Funds4GoodFinanceBorrower",
            program_id,
        )?;

        if expected_borrower_storage_account_pubkey != *borrower_storage_account.key {
            return Err(Funds4GoodError::AccountMismatched.into());
        }
        */

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let Funds4Good_coin_vault_account_data_before =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
            let (pda_Funds4Good_vault, _bump_seed) =
            Pubkey::find_program_address(&[b"Funds4GoodFinance"], program_id);

        if Funds4Good_coin_vault_account_data_before.owner != pda_Funds4Good_vault {
            return Err(Funds4GoodError::Funds4GoodVaultAccountDoesNotMatched.into());
        }
        let Funds4Good_coin_vault_balance_before = Funds4Good_coin_vault_account_data_before.amount;
        let transfer_emi_amount_to_vault_ix = spl_token::instruction::transfer(
            token_program.key,
            borrower_Funds4Good_coin_account_to_debit.key,
            Funds4Good_coin_vault_account.key,
            borrower_main_account.key,
            &[],
            emi_amount_to_pay_input,
        )?;

        msg!("Calling the token program to transfer emi amount to vault...");
        msg!(
            "amount of Funds4Good coin tokens to transfer {}, borrower debit key {}",
            (emi_amount_to_pay_input as f64 / Funds4Good_COIN_DECIMALS as f64),
            borrower_Funds4Good_coin_account_to_debit.key.to_string()
        );

        invoke(
            &transfer_emi_amount_to_vault_ix,
            &[
                borrower_Funds4Good_coin_account_to_debit.clone(),
                Funds4Good_coin_vault_account.clone(),
                borrower_main_account.clone(),
                token_program.clone(),
            ],
        )?;

        let Funds4Good_coin_vault_account_data_after =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_after = Funds4Good_coin_vault_account_data_after.amount;
        msg!(
            "Funds4GoodCoin vault balance after: {}",
            Funds4Good_coin_vault_balance_after
        );
        let vault_balance_increased = Funds4Good_coin_vault_balance_after
            .checked_sub(Funds4Good_coin_vault_balance_before)
            .unwrap();

        if vault_balance_increased != emi_amount_to_pay_input {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        if loan_info_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }

        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
        }

        if loan_info_header_data
            .total_loan_amount
            .checked_div(loan_info_header_data.number_of_emis_needed_to_repay_the_loan as u64)
            .unwrap()
            > emi_amount_to_pay_input
        {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }

        if loan_info_header_data.repaid_amount_by_borrower
            >= loan_info_header_data.total_loan_amount
        {
            return Err(Funds4GoodError::LoanAlreadyPaid.into());
        }
        loan_info_header_data.repaid_amount_by_borrower = loan_info_header_data
            .repaid_amount_by_borrower
            .checked_add(vault_balance_increased)
            .unwrap();

        let loan_info_repayment_timestamp_si = 9116usize
            + (loan_info_header_data.next_index_to_store_repayment_info as usize) * (16usize);
        let loan_info_repayment_timestamp_ei = loan_info_repayment_timestamp_si + 8usize;
        let loan_info_repayment_amount_ei = loan_info_repayment_timestamp_ei + 8usize;
        let now = Clock::get()?.unix_timestamp as u64;
        loan_info_data_byte_array
            [loan_info_repayment_timestamp_si..loan_info_repayment_timestamp_ei]
            .copy_from_slice(&now.to_le_bytes());
        loan_info_data_byte_array[loan_info_repayment_timestamp_ei..loan_info_repayment_amount_ei]
            .copy_from_slice(&vault_balance_increased.to_le_bytes());
        loan_info_header_data.next_index_to_store_repayment_info = loan_info_header_data
            .next_index_to_store_repayment_info
            .checked_add(1u8)
            .unwrap();

            let lenders_data_storage_account = next_account_info(account_info_iter)?;

            if lenders_data_storage_account.owner != program_id {
                return Err(Funds4GoodError::WrongAccountPassed.into());
            }
    
            let mut lenders_storage_data_byte_array =
                lenders_data_storage_account.try_borrow_mut_data()?;
    
            if lenders_storage_data_byte_array[0] != AccTypes::LendersAcc as u8 {
                return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
            }
    
            if lenders_storage_data_byte_array[1] != 1u8 {
                return Err(Funds4GoodError::ExpectedLendersAccNumNotMatched.into());
            }
    
        let emi_amount_distributed_per_lender: u64 = vault_balance_increased.checked_div(loan_info_header_data.next_index_to_store_lender_data as u64).unwrap();

        for i in 0..loan_info_header_data.next_index_to_store_lender_data {
            let loan_info_lender_data_si: usize = state::LOAN_INFO_HEADER_DATA_BYTES
            + (i as usize)
                * state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data_ei: usize =
            loan_info_lender_data_si + state::LOAN_INFO_ACC_LENDER_DATA_BYTES;
        let loan_info_lender_data: LoanInfoAccLendersData =
            state::unpack_to_loan_info_acc_lender_data(
                &loan_info_data_byte_array[loan_info_lender_data_si..loan_info_lender_data_ei],
            )
            .unwrap();
        

        // lender_id_input can vary from 0 to 49_999 included
        let lender_si_in_lenders_data_byte_array: usize = 2usize
            + (loan_info_lender_data.lender_id as usize)
                .checked_mul(state::LENDER_ACC_DATA_SIZE)
                .unwrap();
        let lender_ei_in_lenders_data_byte_array: usize =
            lender_si_in_lenders_data_byte_array + state::LENDER_ACC_DATA_SIZE;
        let mut lender_acc_data: LenderAccountData = state::unpack_to_lender_account_data(
            &lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        // update lender data in LendersAccountDataArray
        lender_acc_data.total_lending_amount = lender_acc_data
            .total_lending_amount
            .checked_add(vault_balance_increased as u128)
            .unwrap();
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_add(vault_balance_increased)
            .unwrap();

        lender_acc_data.amount_to_withdraw_or_lend = lender_acc_data.amount_to_withdraw_or_lend.checked_add(emi_amount_distributed_per_lender).unwrap();

        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();
        }

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

}