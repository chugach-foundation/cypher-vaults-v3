use anchor_lang::{InstructionData, ToAccountMetas};
use anchor_spl::token;
use cypher_client::utils::{derive_account_address, derive_sub_account_address};
use solana_program_test::{processor, BanksClientError};
use solana_sdk::{
    pubkey::Pubkey, rent, signature::Keypair, signer::Signer, system_program, sysvar::SysvarId,
};
use vaults::{CreateVaultArgs, OpenDepositsArgs, Vault, LP_TOKEN_SEED, VAULT_SEED};

use super::{ProgramTestContext, ProgramTestContextConfig};

pub struct UserCookie {
    pub keypair: Keypair,
}

impl UserCookie {
    pub fn new(test: &mut ProgramTestContext) -> Result<UserCookie, BanksClientError> {
        let keypair = Keypair::new();
        test.add_account(&keypair.pubkey());
        Ok(UserCookie { keypair })
    }
}

pub struct VaultCookie {
    pub authority: Keypair,
    pub address: Pubkey,
}

impl VaultCookie {
    pub async fn create_vault(
        test: &mut ProgramTestContext,
        id: u64,
    ) -> Result<VaultCookie, BanksClientError> {
        let authority = Keypair::new();
        let (cypher_account, cypher_account_bump) = derive_account_address(&authority.pubkey(), 0);
        let (cypher_sub_account, cypher_sub_account_bump) =
            derive_sub_account_address(&cypher_account, 0);

        let (vault, vault_bump) = VaultCookie::derive_vault_address(&authority.pubkey(), id);

        let accounts = vaults::accounts::CreateVault {
            vault,
            clearing: test.clearing,
            cypher_account,
            cypher_sub_account,
            authority: authority.pubkey(),
            payer: authority.pubkey(),
            system_program: system_program::id(),
            cypher_program: cypher_client::id(),
            rent: rent::Rent::id(),
        };

        let ix_data = vaults::instruction::CreateVault {
            args: CreateVaultArgs {
                id,
                account_number: 0,
                account_bump: cypher_account_bump,
                sub_account_number: 0,
                sub_account_bump: cypher_sub_account_bump,
                sub_account_alias: [0; 32],
                token_info_count: 1,
            },
        }
        .data();

        test.send_anchor_ix(vaults::id(), &accounts, ix_data, Some(&[&authority]))
            .await?;

        Ok(VaultCookie {
            authority,
            address: vault,
        })
    }

    pub async fn open_deposits(
        &self,
        test: &mut ProgramTestContext,
        token_mint: Pubkey,
    ) -> Result<(), BanksClientError> {
        let (lp_token_mint, _) = VaultCookie::derive_lp_mint_address(&self.address, &token_mint);

        let accounts = vaults::accounts::OpenDeposits {
            vault: self.address,
            lp_token_mint,
            authority: self.authority.pubkey(),
            payer: self.authority.pubkey(),
            system_program: system_program::id(),
            token_program: token::ID,
            rent: rent::Rent::id(),
        };

        let ix_data = vaults::instruction::OpenDeposits {
            args: OpenDepositsArgs {
                token_mint,
                deposit_limit: u64::MAX,
                decimals: 0,
            },
        }
        .data();

        test.send_anchor_ix(vaults::id(), &accounts, ix_data, Some(&[&self.authority]))
            .await?;

        Ok(())
    }

    fn derive_vault_address(authority: &Pubkey, id: u64) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[VAULT_SEED, authority.as_ref(), id.to_le_bytes().as_ref()],
            &vaults::id(),
        )
    }

    fn derive_lp_mint_address(vault: &Pubkey, token_mint: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[LP_TOKEN_SEED, vault.as_ref(), token_mint.as_ref()],
            &vaults::id(),
        )
    }
}

pub async fn init_new_test() -> Result<(ProgramTestContext), BanksClientError> {
    let config = ProgramTestContextConfig {
        mint_decimals: vec![],
    };
    let test = ProgramTestContext::start_new(&config).await;

    let cache_account = Keypair::new();

    Ok((test))
}
