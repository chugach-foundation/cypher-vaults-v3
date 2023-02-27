use anchor_lang::prelude::*;

#[error_code]
#[derive(PartialEq, Eq)]
pub enum ErrorCode {
    #[msg("Default error code.")]
    Default,

    #[msg("The given vault does not support this operation.")]
    InvalidVaultType,

    #[msg("The given token mint is not supported.")]
    InvalidTokenMint,

    #[msg("The given token has existing deposits.")]
    TokenWithDeposits,

    #[msg("The given token has outstanding LP token supply.")]
    TokenWithLpSupply,
}

#[macro_export]
macro_rules! check {
    ($invariant:expr, $error:tt $(,)?) => {
        if !($invariant) {
            #[cfg(test)]
            anchor_lang::solana_program::msg!(
                "{} at line {} in {}",
                $crate::error::ErrorCode::$error,
                line!(),
                file!()
            );
            return Err($crate::error::ErrorCode::$error.into());
        }
    };
    ($invariant:expr, $error:expr $(,)?) => {
        if !($invariant) {
            #[cfg(test)]
            anchor_lang::solana_program::msg!("{} at line {} in {}", $error, line!(), file!());
            return Err($error.into());
        }
    };
}
