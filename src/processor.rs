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

    fn initialize_loan_info_account(
        accounts: &[AccountInfo],
        num_days_left_for_first_repayment_input: u16,
        num_emis_needed_to_repay_the_loan_input: u16,
        num_days_for_fundraising_input: u16,
        total_loan_amount_input: u64,
        program_id: &Pubkey,
    ) -> ProgramResult {
        // also update borrower storage account
        let account_info_iter = &mut accounts.iter();
        let guarantor_main_account = next_account_info(account_info_iter)?;

        if !guarantor_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }
       let borrower_main_account = next_account_info(account_info_iter)?;

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        // just for extra safety, even this check is not required
        if loan_info_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }

        let rent = Rent::get()?;

        if !rent.is_exempt(
            loan_info_storage_account.lamports(),
            loan_info_storage_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }

        if loan_info_storage_account.data_len() != state::LOAN_INFO_ACC_DATA_SIZE {
            return Err(Funds4GoodError::DataSizeNotMatched.into());
        }

        let mut loan_info_data_byte_array = loan_info_storage_account.data.borrow_mut();

        if loan_info_data_byte_array[0] != 0 {
            return Err(Funds4GoodError::LoanInfoDataAlreadyInitialized.into());
        }

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

        let mut borrower_data =
            BorrowerAccount::unpack(&borrower_storage_account.data.try_borrow().unwrap())?;
        if borrower_data.is_active_loan != 0 {
            return Err(Funds4GoodError::BorrowerAlreadyHaveActiveLoan.into());
        }
        if borrower_data.acc_type != AccTypes::BorrowerAcc as u8 {
            return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
        }
        borrower_data.active_loan_address = *loan_info_storage_account.key;
        BorrowerAccount::pack(
            borrower_data,
            &mut borrower_storage_account.data.try_borrow_mut().unwrap(),
        )?;

        let num_seconds_in_one_day: u64 = 86400u64;
        let now = Clock::get()?.unix_timestamp as u64;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_unchecked_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();
        loan_info_header_data.acc_type = AccTypes::LoanInfoAcc as u8;
        loan_info_header_data.borrower_main_acc_pubkey = *borrower_main_account.key;
        loan_info_header_data.guarantor_main_acc_pubkey = *guarantor_main_account.key;
        loan_info_header_data.loan_approval_timestamp = now.clone();
        let calculate_fundraising_period_ending_timestamp: u64 = now
            .checked_add(
                (num_days_for_fundraising_input as u64)
                    .checked_mul(num_seconds_in_one_day)
                    .unwrap(),
            )
            .unwrap();
        loan_info_header_data.fundraising_period_ending_timestamp =
            calculate_fundraising_period_ending_timestamp;
        // a user can pay upto 5 days late, after that his credit score will decrease
        loan_info_header_data.first_repayment_last_date_timestamp =
            (num_days_left_for_first_repayment_input
                .checked_add(5u16)
                .unwrap() as u64)
                .checked_mul(num_seconds_in_one_day)
                .unwrap();
        loan_info_header_data.total_loan_amount = total_loan_amount_input;
        loan_info_header_data.number_of_emis_needed_to_repay_the_loan =
            num_emis_needed_to_repay_the_loan_input as u8;

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    // This will credit all free funds to lender wallet
    fn process_withdraw_lender_free_wallet_funds(
        accounts: &[AccountInfo],
        lender_id_input: u32,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let lender_main_account = next_account_info(account_info_iter)?;

        if !lender_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lender_Funds4Good_coin_account_to_credit = next_account_info(account_info_iter)?;

        // Funds4Good_coin_vault_account is program controlled vault account and can be only controlled by our deployed program for debit funds or any kind of operation
        let Funds4Good_coin_vault_account = next_account_info(account_info_iter)?;

        /*
        check if right vault is passed 
        */

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

        // lender_id_input can vary from 0 to 49_999 included
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

        if lender_acc_data.is_account_active != 1u8
            && lender_acc_data.lender_main_acc_pubkey != *lender_main_account.key
        {
            return Err(Funds4GoodError::InvalidLenderIdInput.into());
        }

        let withdraw_amount: u64 = lender_acc_data.amount_to_withdraw_or_lend;
        lender_acc_data.total_unique_lending_amount = lender_acc_data
            .total_unique_lending_amount
            .checked_sub(withdraw_amount.clone())
            .unwrap();
        lender_acc_data.amount_to_withdraw_or_lend = 0u64;

        state::pack_to_lender_account_data(
            lender_acc_data,
            &mut lenders_storage_data_byte_array
                [lender_si_in_lenders_data_byte_array..lender_ei_in_lenders_data_byte_array],
        )
        .unwrap();

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let Funds4Good_coin_vault_account_data_before =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_before = Funds4Good_coin_vault_account_data_before.amount;

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"Funds4GoodFinance"], program_id);
        if pda != *pda_account.key {
            return Err(Funds4GoodError::PdaAccountDoesNotMatched.into());
        }

        let transfer_withdraw_amount_to_lender_ix = spl_token::instruction::transfer(
            token_program.key,
            Funds4Good_coin_vault_account.key,
            lender_Funds4Good_coin_account_to_credit.key,
            &pda,
            &[&pda],
            withdraw_amount,
        )?;
        msg!("Calling the token program to transfer withdraw amount to lender...");
        msg!(
            "amount of Funds4Good coin tokens to transfer {}, lender credit key {}",
            (withdraw_amount as f64 / Funds4Good_COIN_DECIMALS as f64),
            lender_Funds4Good_coin_account_to_credit.key.to_string()
        );
        invoke_signed(
            &transfer_withdraw_amount_to_lender_ix,
            &[
                Funds4Good_coin_vault_account.clone(),
                lender_Funds4Good_coin_account_to_credit.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"Funds4GoodFinance"[..], &[bump_seed]]],
        )?;

        let Funds4Good_coin_vault_account_data_after =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_after = Funds4Good_coin_vault_account_data_after.amount;
        msg!(
            "Funds4GoodCoin vault balance after: {}",
            Funds4Good_coin_vault_balance_after
        );

        let vault_balance_decreased = Funds4Good_coin_vault_balance_before
            .checked_sub(Funds4Good_coin_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != withdraw_amount {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }

        Ok(())
    }

    fn process_withdraw_collected_loan_funds(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let borrower_main_account = next_account_info(account_info_iter)?;

        if !borrower_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let borrower_Funds4Good_ata_to_credit = next_account_info(account_info_iter)?;

        let Funds4Good_coin_vault_account = next_account_info(account_info_iter)?;

        /*
        let Funds4Good_vault_pubkey = utils::get_Funds4Good_vault_pubkey();

        if Funds4Good_vault_pubkey != *Funds4Good_coin_vault_account.key {
            return Err(Funds4GoodError::Funds4GoodVaultAccountDoesNotMatched.into());
        }

        */

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let loan_info_storage_account = next_account_info(account_info_iter)?;

        if loan_info_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }

        // update lender payment in LoanInfoAccData
        let mut loan_info_data_byte_array = loan_info_storage_account.try_borrow_mut_data()?;
        let mut loan_info_header_data: LoanInfoAccDataHeader = state::unpack_to_loan_info_header(
            &loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        if loan_info_header_data.acc_type != AccTypes::LoanInfoAcc as u8 {
            return Err(Funds4GoodError::ExpectedAccountTypeMismatched.into());
        }

        if loan_info_header_data.total_amount_lended < loan_info_header_data.total_loan_amount {
            return Err(Funds4GoodError::CollectedLoanFundsAlreadyWithdrawn.into());
        }

        if loan_info_header_data.borrower_main_acc_pubkey != *borrower_main_account.key {
            return Err(Funds4GoodError::BorrowerAccountMismatched.into());
        }

        // borrower can withdraw total_amount_lended, after withdrawing we will set it to 0, so that he can't withdraw second time. We can also use a extra variable to store if funds
        // withdrawn or not

        let Funds4Good_coin_vault_account_data_before =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_before = Funds4Good_coin_vault_account_data_before.amount;

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"Funds4GoodFinance"], program_id);
        if pda != *pda_account.key {
            return Err(Funds4GoodError::PdaAccountDoesNotMatched.into());
        }

        let transfer_collected_loan_funds_to_borrower_ix = spl_token::instruction::transfer(
            token_program.key,
            Funds4Good_coin_vault_account.key,
            borrower_Funds4Good_ata_to_credit.key,
            &pda,
            &[&pda],
            loan_info_header_data.total_amount_lended,
        )?;
        msg!("Calling the token program to transfer collected loan amount to borrower...");
        msg!(
            "amount of Funds4Good coin tokens to transfer {}, lender credit key {}",
            (loan_info_header_data.total_amount_lended as f64 / Funds4Good_COIN_DECIMALS as f64),
            borrower_Funds4Good_ata_to_credit.key.to_string()
        );
        invoke_signed(
            &transfer_collected_loan_funds_to_borrower_ix,
            &[
                Funds4Good_coin_vault_account.clone(),
                borrower_Funds4Good_ata_to_credit.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"Funds4GoodFinance"[..], &[bump_seed]]],
        )?;

        let Funds4Good_coin_vault_account_data_after =
            TokenAccount::unpack(&Funds4Good_coin_vault_account.data.borrow())?;
        let Funds4Good_coin_vault_balance_after = Funds4Good_coin_vault_account_data_after.amount;
        msg!(
            "Funds4GoodCoin vault balance after: {}",
            Funds4Good_coin_vault_balance_after
        );

        let vault_balance_decreased = Funds4Good_coin_vault_balance_before
            .checked_sub(Funds4Good_coin_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != loan_info_header_data.total_amount_lended {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }

        loan_info_header_data.total_amount_lended = 0u64;

        state::pack_to_loan_info_header(
            loan_info_header_data,
            &mut loan_info_data_byte_array
                [state::LOAN_INFO_HEADER_START_INDEX..state::LOAN_INFO_HEADER_END_INDEX],
        )
        .unwrap();

        Ok(())
    }

    fn process_transfer_Funds4Good_vault_account_ownership(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let Funds4Good_vault_account = next_account_info(account_info_iter)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b"Funds4GoodFinance"], program_id);

        let rent = Rent::get()?;

        if !rent.is_exempt(
            Funds4Good_vault_account.lamports(),
            Funds4Good_vault_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }
        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let Funds4Good_vault_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            Funds4Good_vault_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;

        msg!("Calling the token program to transfer Funds4Good vault account ownership to program...");
        invoke(
            &Funds4Good_vault_owner_change_ix,
            &[
                Funds4Good_vault_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_initialize_lenders_storage_account(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let lenders_storage_account = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        if !rent.is_exempt(
            lenders_storage_account.lamports(),
            lenders_storage_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }

        if lenders_storage_account.data_len() != state::LENDERS_STORAGE_ACC_DATA_SIZE {
            return Err(Funds4GoodError::DataSizeNotMatched.into());
        }

        let mut lenders_storage_data_byte_array =
            lenders_storage_account.data.try_borrow_mut().unwrap();

        if lenders_storage_data_byte_array[0] != 0 {
            return Err(Funds4GoodError::LendersStorageDataAlreadyInitialized.into());
        }

        lenders_storage_data_byte_array[0] = AccTypes::LendersAcc as u8;
        // currently for prototype I'm making every lenders_data_storage_acc_number to 1, but in future when we need more accounts, we have to increment it for every new account generation
        lenders_storage_data_byte_array[1] = 1u8;

        Ok(())
    }

    fn process_initialize_guarantor_storage_account(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let guarantor_main_account = next_account_info(account_info_iter)?;
        if !guarantor_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let guarantor_storage_account = next_account_info(account_info_iter)?;
        if guarantor_storage_account.owner != program_id {
            return Err(Funds4GoodError::WrongAccountPassed.into());
        }

        /*
        let expected_guarantor_storage_account_pubkey = Pubkey::create_with_seed(
            guarantor_main_account.key,
            "Funds4GoodFinanceGuarantor",
            program_id,
        )?;

        if expected_guarantor_storage_account_pubkey != *guarantor_storage_account.key {
            return Err(Funds4GoodError::AccountMismatched.into());
        }
        */
        let rent = Rent::get()?;
        if !rent.is_exempt(
            guarantor_storage_account.lamports(),
            guarantor_storage_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }

        // put a condition if guarantor_storage_account.data_len() != 75, then error

        let mut guarantor_data =
            GuarantorAccount::unpack_unchecked(&guarantor_storage_account.data.borrow())?;

        if guarantor_data.is_initialized() {
            return Err(Funds4GoodError::GuarantorAccountAlreadyInitialized.into());
        }

        guarantor_data.is_initialized = true;
        guarantor_data.acc_type = AccTypes::GuarantorAcc as u8;
        guarantor_data.guarantor_main_acc_pubkey = *guarantor_main_account.key;
        guarantor_data.approval_score = 500_000_000_000u64;

        GuarantorAccount::pack(
            guarantor_data,
            &mut guarantor_storage_account.data.borrow_mut(),
        )?;

        Ok(())
    }

    //On each airdrop, users will get 250 Funds4GoodCoin test tokens. A user can airdrop a maximum of 10 times.
    fn process_airdrop_users_with_Funds4Good_test_coins(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let amount_to_airdrop = 500_000_000_000u64;
        let max_amount_to_airdrop = 2_500_000_000_000u64;

        let account_info_iter = &mut accounts.iter();
        let airdrop_user_main_account = next_account_info(account_info_iter)?;

        if !airdrop_user_main_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let Funds4Good_coin_airdrop_user_storage_account = next_account_info(account_info_iter)?;

        let rent = Rent::get()?;

        if !rent.is_exempt(
            Funds4Good_coin_airdrop_user_storage_account.lamports(),
            Funds4Good_coin_airdrop_user_storage_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }

        let expected_Funds4Good_coin_airdrop_user_storage_pubkey = Pubkey::create_with_seed(
            airdrop_user_main_account.key,
            "Funds4GoodFinanceAirdrop",
            program_id,
        )?;

        if expected_Funds4Good_coin_airdrop_user_storage_pubkey
            != *Funds4Good_coin_airdrop_user_storage_account.key
        {
            return Err(Funds4GoodError::AccountMismatched.into());
        }

        let mut Funds4Good_coin_airdrop_user_storage_byte_array =
            Funds4Good_coin_airdrop_user_storage_account.try_borrow_mut_data()?;
        let stored_amount = u64::from_le_bytes(
            Funds4Good_coin_airdrop_user_storage_byte_array[0..8]
                .try_into()
                .unwrap(),
        );
        if stored_amount >= max_amount_to_airdrop {
            return Err(Funds4GoodError::UserAlreadyAirdroped.into());
        }
        let new_total_airdrop_amount_for_user: u64 =
            stored_amount.checked_add(amount_to_airdrop).unwrap();
        Funds4Good_coin_airdrop_user_storage_byte_array
            .copy_from_slice(&new_total_airdrop_amount_for_user.to_le_bytes());

        let user_Funds4Good_coin_associated_token_to_credit_account =
            next_account_info(account_info_iter)?;

        let airdrop_vault_Funds4Good_coin_account = next_account_info(account_info_iter)?;

        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let pda_account = next_account_info(account_info_iter)?;

        // we can also store bump_seed to save computations
        let (pda, bump_seed) = Pubkey::find_program_address(&[b"Funds4GoodFinanceAirdrop"], program_id);
        if pda != *pda_account.key {
            return Err(Funds4GoodError::PdaAccountDoesNotMatched.into());
        }

        let airdrop_vault_account_data =
            TokenAccount::unpack(&airdrop_vault_Funds4Good_coin_account.data.borrow())?;

        let airdrop_vault_balance_before = airdrop_vault_account_data.amount;

        let transfer_Funds4Good_coin_to_airdroper_ix = spl_token::instruction::transfer(
            token_program.key,
            airdrop_vault_Funds4Good_coin_account.key,
            user_Funds4Good_coin_associated_token_to_credit_account.key,
            &pda,
            &[&pda],
            amount_to_airdrop,
        )?;
        msg!("Calling the token program to transfer airdrop tokens to the user...");
        invoke_signed(
            &transfer_Funds4Good_coin_to_airdroper_ix,
            &[
                airdrop_vault_Funds4Good_coin_account.clone(),
                user_Funds4Good_coin_associated_token_to_credit_account.clone(),
                pda_account.clone(),
                token_program.clone(),
            ],
            &[&[&b"Funds4GoodFinanceAirdrop"[..], &[bump_seed]]],
        )?;

        let airdrop_vault_account_data_after =
            TokenAccount::unpack(&airdrop_vault_Funds4Good_coin_account.data.borrow())?;
        let airdrop_vault_balance_after = airdrop_vault_account_data_after.amount;
        let vault_balance_decreased = airdrop_vault_balance_before
            .checked_sub(airdrop_vault_balance_after)
            .unwrap();

        if vault_balance_decreased != amount_to_airdrop {
            return Err(Funds4GoodError::ExpectedAmountMismatch.into());
        }

        Ok(())
    }

    fn process_transfer_airdrop_vault_account_ownership(
        accounts: &[AccountInfo],
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();
        let initializer_account = next_account_info(account_info_iter)?;

        if !initializer_account.is_signer {
            return Err(ProgramError::MissingRequiredSignature);
        }

        let airdrop_vault_Funds4Good_coin_account = next_account_info(account_info_iter)?;

        let (pda, _nonce) = Pubkey::find_program_address(&[b"Funds4GoodFinanceAirdrop"], program_id);

        let rent = Rent::get()?;

        if !rent.is_exempt(
            airdrop_vault_Funds4Good_coin_account.lamports(),
            airdrop_vault_Funds4Good_coin_account.data_len(),
        ) {
            return Err(Funds4GoodError::NotRentExempt.into());
        }
        let token_program = next_account_info(account_info_iter)?;

        if token_program.key != &spl_token::id() {
            return Err(Funds4GoodError::InvalidTokenProgram.into());
        }

        let airdrop_vault_owner_change_ix = spl_token::instruction::set_authority(
            token_program.key,
            airdrop_vault_Funds4Good_coin_account.key,
            Some(&pda),
            spl_token::instruction::AuthorityType::AccountOwner,
            initializer_account.key,
            &[&initializer_account.key],
        )?;

        msg!("Calling the token program to transfer Funds4Good airdrop vault account ownership to program...");
        invoke(
            &airdrop_vault_owner_change_ix,
            &[
                airdrop_vault_Funds4Good_coin_account.clone(),
                initializer_account.clone(),
                token_program.clone(),
            ],
        )?;

        Ok(())
    }

    fn process_close_loan_info_account(accounts: &[AccountInfo]) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }

    fn process_return_funds_to_lenders(
        accounts: &[AccountInfo],
        num_accounts_input: u16,
        program_id: &Pubkey,
    ) -> ProgramResult {
        let account_info_iter = &mut accounts.iter();

        Ok(())
    }
}
