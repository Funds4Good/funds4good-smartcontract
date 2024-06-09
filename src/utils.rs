use solana_program::{msg, pubkey::Pubkey};
use std::convert::TryInto;


pub enum AccTypes {
    BorrowerAcc = 2,
    LendersAcc = 3,
    GuarantorAcc = 4,
    LoanInfoAcc = 5,
}


pub fn get_admin_pubkey() -> Pubkey {
    let admin_pubkey_str = "857Tm9dNi6Ypur9zCcJ9oAhqYd3bE6J6s2ww77PKCSa";
    let pubkey_vec = bs58::decode(admin_pubkey_str).into_vec().unwrap();
    let admin_pubkey = Pubkey::new(&pubkey_vec);
    return admin_pubkey;
}