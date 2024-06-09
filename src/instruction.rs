use crate::error::Funds4GoodError::InvalidInstruction;
use solana_program::msg;
use solana_program::program_error::ProgramError;
use std::convert::TryInto;

pub enum Funds4GoodInstruction {
    LendToBorrower {
        amount_to_lend_input: u64,
        lender_id_input: u32,
    },

    WithdrawLenderFreeWalletFunds { lender_id_input: u32 },
    WithdrawCollectedLoanFunds {},
    TransferFunds4GoodVaultAccountOwnership {},
    InitializeLendersStorageAccount {},
    InitializeGuarantorAccount {},
    InitializeBorrowerAccount {},
    PayEMIforLoan { emi_amount_to_pay_input: u64 },
    InitializeLoanInfoAccount {
        num_days_left_for_first_repayment_input: u16,
        num_emis_needed_to_repay_the_loan_input: u16,
        num_days_for_fundraising_input: u16,
        total_loan_amount_input: u64,
    },
    AirdropUsersWithFunds4GoodTestCoins {},
    TransferAirdropVaultAccountOwnership {},
    ReturnFundsToLenders { num_accounts_input: u16 },
    CloseLoanInfoAccount {},
}

impl Funds4GoodInstruction {
    pub fn unpack(input: &[u8]) -> Result<Self, ProgramError> {
        msg!("In {}", input.len());

        Ok(match input[0] {
            0 => Self::LendToBorrower {
                amount_to_lend_input: Self::unpack_to_u64(&input[1..9])?,
                lender_id_input: Self::unpack_to_u32(&input[9..13]),
            },
            1 => Self::WithdrawLenderFreeWalletFunds {
                lender_id_input: Self::unpack_to_u32(&input[1..5]),
            },
            2 => Self::WithdrawCollectedLoanFunds {},
            3 => Self::TransferFunds4GoodVaultAccountOwnership {},
            4 => Self::InitializeLendersStorageAccount {},
            5 => Self::InitializeBorrowerAccount {},
            6 => Self::InitializeGuarantorAccount {},
            7 => Self::PayEMIforLoan {
                emi_amount_to_pay_input: Self::unpack_to_u64(&input[1..9])?,
            },
            8 => Self::InitializeLoanInfoAccount {
                num_days_left_for_first_repayment_input: Self::unpack_to_u16(&input[1..3]),
                num_emis_needed_to_repay_the_loan_input: Self::unpack_to_u16(&input[3..5]),
                num_days_for_fundraising_input: Self::unpack_to_u16(&input[5..7]),
                total_loan_amount_input: Self::unpack_to_u64(&input[7..15])?,
            },
            9 => Self::AirdropUsersWithFunds4Good
    TestCoins {},
            10 => Self::TransferAirdropVaultAccountOwnership {},
            11 => Self::ReturnFundsToLenders {
                num_accounts_input: Self::unpack_to_u16(&input[1..3]),
            },
            12 => Self::CloseLoanInfoAccount {},
            _ => return Err(InvalidInstruction.into()),
        })
    }

    fn unpack_to_u64(input: &[u8]) -> Result<u64, ProgramError> {
        msg!("in unpack");
        let value = input
            .get(..8)
            .and_then(|slice| slice.try_into().ok())
            .map(u64::from_le_bytes)
            .ok_or(InvalidInstruction)?;
        msg!("unpack value {}", value);
        Ok(value)
    }

    fn unpack_to_u16(input: &[u8]) -> u16 {
        (input[0] as u16) | (input[1] as u16) << 8
    }

    pub fn unpack_to_u32(input: &[u8]) -> u32 {
        let amount = (input[0] as u32)
            | (input[1] as u32) << 8
            | (input[2] as u32) << 16
            | (input[3] as u32) << 24;
        msg!("u32 unpack amount {}", amount);
        return amount;
    }
}
