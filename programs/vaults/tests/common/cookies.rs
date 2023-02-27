use cypher_client::utils::{derive_account_address, derive_sub_account_address};
use solana_program_test::BanksClientError;
use solana_sdk::{pubkey::Pubkey, signature::Keypair, signer::Signer};
use vaults::{vaults, Vault, VAULT_SEED};

use super::{VaultProgramTest, VaultProgramTestConfig};

#[derive(Default)]
pub struct VaultCookie {
    pub authority: Keypair,
    pub address: Pubkey,
}

impl VaultCookie {
    pub async fn create_vault(
        test: &mut VaultProgramTest,
    ) -> Result<VaultCookie, BanksClientError> {
        let authority = Keypair::new();
        let (cypher_account, cypher_account_bump) = derive_account_address(&authority.pubkey(), 0);
        let (cypher_sub_account, cypher_sub_account_bump) =
            derive_sub_account_address(&cypher_account, 0);

        let vault_address = Vault::test
            .send_anchor_ix(program_id, accounts, ix_data, signers)
            .await?;

        Ok(VaultCookie {
            authority,
            address: vault_address,
        })
    }

    fn derive_vault_address(authority: &Pubkey, id: u64) -> Pubkey {
        Pubkey::find_program_address(
            &[VAULT_SEED, authority.as_ref(), id.to_le_bytes().as_ref()],
            &vaults::id(),
        )
        .0
    }
}

pub async fn init_new_test() -> Result<(VaultProgramTest, VaultCookie), BanksClientError> {
    let config = VaultProgramTestConfig::default();
    let test = VaultProgramTest::start_new(&config).await;

    let cookie = VaultCookie::default();

    Ok((test, cookie))
}
