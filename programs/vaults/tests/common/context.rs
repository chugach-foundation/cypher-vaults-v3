use anchor_lang::{AccountDeserialize, AnchorSerialize, ToAccountMetas};
use anchor_spl::{associated_token, token::spl_token};
use bincode::deserialize;
use bytemuck::from_bytes;

use solana_program::{
    account_info::AccountInfo,
    clock::{Clock, UnixTimestamp},
    program_option::COption,
    program_pack::Pack,
    pubkey::*,
    rent::*,
    sysvar,
};
use solana_program_test::*;
use solana_sdk::{
    account::{AccountSharedData, WritableAccount},
    instruction::Instruction,
    signature::{Keypair, Signer},
    system_instruction,
    transaction::Transaction,
};
use spl_token::{state::*, *};

use bytemuck::Pod;

trait AddPacked {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    );
}

impl AddPacked for ProgramTest {
    fn add_packable_account<T: Pack>(
        &mut self,
        pubkey: Pubkey,
        amount: u64,
        data: &T,
        owner: &Pubkey,
    ) {
        let mut account = solana_sdk::account::Account::new(amount, T::get_packed_len(), owner);
        data.pack_into_slice(&mut account.data);
        self.add_account(pubkey, account);
    }
}

#[derive(Default)]
pub struct VaultProgramTestConfig {
    pub mint_decimals: Vec<u8>,
}

pub struct VaultProgramTest {
    pub context: ProgramTestContext,
    pub rent: Rent,

    pub mint_auth: Keypair,
    pub mint_list: Vec<Pubkey>,
}

impl VaultProgramTest {
    #[allow(dead_code)]
    pub async fn start_new(config: &VaultProgramTestConfig) -> Self {
        let test = ProgramTest::new("lending", lending::id(), processor!(lending::entry));

        //test.set_compute_max_units(1_400_000);

        VaultProgramTest::start_new_with_program_test(test, config).await
    }

    #[allow(dead_code)]
    pub async fn start_new_with_program_test(
        mut test: ProgramTest,
        config: &VaultProgramTestConfig,
    ) -> Self {
        solana_logger::setup_with_default(
            "solana_rbpf::vm=info,\
              solana_runtime::message_processor=trace,
              solana_runtime::system_instruction_processor=info,\
              solana_program_test=info",
        );

        test.add_program("cypher", cypher_client::id(), None);

        let mint_auth = Keypair::new();

        let mut mint_list = vec![];

        // add native mint
        test.add_packable_account(
            native_mint::id(),
            u32::MAX as u64,
            &Mint {
                is_initialized: true,
                mint_authority: COption::Some(mint_auth.pubkey()),
                decimals: 9,
                ..Mint::default()
            },
            &spl_token::id(),
        );

        // add mints
        for i in config.mint_decimals {
            let mint_kp = Keypair::new();
            test.add_packable_account(
                mint_kp.pubkey(),
                u32::MAX as u64,
                &Mint {
                    is_initialized: true,
                    mint_authority: COption::Some(mint_auth.pubkey()),
                    decimals: i,
                    ..Mint::default()
                },
                &spl_token::id(),
            );
            mint_list.push(mint_kp.pubkey());
        }

        test.add_account(
            mint_auth.pubkey(),
            solana_sdk::account::Account::new(
                u32::MAX as u64,
                0,
                &solana_sdk::system_program::id(),
            ),
        );

        let lending_id = lending::id();

        let mut context = test.start_with_context().await;
        let rent = context.banks_client.get_rent().await.unwrap();

        VaultProgramTest {
            context,
            rent,
            mint_auth,
            mint_list,
        }
    }

    #[allow(dead_code)]
    pub async fn process_transaction(
        &mut self,
        instructions: &[Instruction],
        signers: Option<&[&Keypair]>,
    ) -> Result<(), BanksClientError> {
        let mut transaction =
            Transaction::new_with_payer(instructions, Some(&self.context.payer.pubkey()));

        let mut all_signers = vec![&self.context.payer];

        if let Some(signers) = signers {
            all_signers.extend_from_slice(signers);
        }

        // This fails when warping is involved - https://gitmemory.com/issue/solana-labs/solana/18201/868325078
        // let recent_blockhash = self.context.banks_client.get_recent_blockhash().await.unwrap();

        transaction.sign(&all_signers, self.context.last_blockhash);

        self.context
            .banks_client
            .process_transaction(transaction)
            .await
    }

    #[allow(dead_code)]
    pub async fn get_account(&mut self, address: Pubkey) -> solana_sdk::account::Account {
        self.context
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap()
    }

    #[allow(dead_code)]
    pub fn get_payer_pk(&self) -> Pubkey {
        self.context.payer.pubkey()
    }

    #[allow(dead_code)]
    pub async fn get_token_account(&mut self, address: Pubkey) -> spl_token::state::Account {
        let token = self
            .context
            .banks_client
            .get_account(address)
            .await
            .unwrap()
            .unwrap();
        spl_token::state::Account::unpack(&token.data[..]).unwrap()
    }

    #[allow(dead_code)]
    pub async fn get_token_balance(&mut self, address: Pubkey) -> u64 {
        self.get_token_account(address).await.amount
    }

    #[allow(dead_code)]
    pub async fn load_account_result(
        &mut self,
        acc_pk: Pubkey,
    ) -> Result<Option<solana_sdk::account::Account>, BanksClientError> {
        self.context.banks_client.get_account(acc_pk).await
    }

    #[allow(dead_code)]
    pub async fn load_account<T: Pod>(&mut self, acc_pk: Pubkey) -> Box<T> {
        let mut acc = self
            .context
            .banks_client
            .get_account(acc_pk)
            .await
            .unwrap()
            .unwrap();
        let acc_info: AccountInfo = (&acc_pk, &mut acc).into();

        let data = &acc_info.try_borrow_data().unwrap()[8..];

        let col: Vec<u8> = data.to_vec();
        let data = col.as_slice();

        let parsed: &T = from_bytes(data);
        Box::new(*parsed)
    }

    #[allow(dead_code)]
    pub async fn send_anchor_ix(
        &mut self,
        program_id: Pubkey,
        accounts: &(dyn ToAccountMetas + Send + Sync),
        ix_data: Vec<u8>,
        signers: Option<&[&Keypair]>,
    ) -> Result<(), BanksClientError> {
        self.send_anchor_ix_with_compute(program_id, accounts, ix_data, signers, false)
            .await
    }

    #[allow(dead_code)]
    pub async fn send_anchor_ix_with_compute(
        &mut self,
        program_id: Pubkey,
        accounts: &(dyn ToAccountMetas + Send + Sync),
        ix_data: Vec<u8>,
        signers: Option<&[&Keypair]>,
        increase_cu: bool,
    ) -> Result<(), BanksClientError> {
        let ix = Instruction {
            program_id,
            data: ix_data,
            accounts: accounts
                .to_account_metas(None)
                .into_iter()
                .map(|mut meta| {
                    if meta.pubkey == self.get_payer_pk()
                        || signers.is_some()
                            && signers.unwrap().iter().any(|k| meta.pubkey == k.pubkey())
                    {
                        meta.is_signer = true;
                    }
                    meta
                })
                .collect(),
        };

        let mut ixs = vec![];

        if increase_cu {
            ixs.extend(vec![
                Instruction::new_with_borsh(
                    solana_sdk::compute_budget::id(),
                    &solana_sdk::compute_budget::ComputeBudgetInstruction::SetComputeUnitLimit(
                        1_400_000,
                    ),
                    vec![],
                ),
                Instruction::new_with_borsh(
                    solana_sdk::compute_budget::id(),
                    &solana_sdk::compute_budget::ComputeBudgetInstruction::SetComputeUnitPrice(1),
                    vec![],
                ),
            ]);
        }
        ixs.push(ix);

        self.process_transaction(ixs.as_slice(), signers).await
    }

    #[allow(dead_code)]
    pub async fn transfer_lamports(
        &mut self,
        dst: Pubkey,
        lamports: u64,
    ) -> Result<(), BanksClientError> {
        self.process_transaction(
            &[solana_sdk::system_instruction::transfer(
                &self.get_payer_pk(),
                &dst,
                lamports,
            )],
            None,
        )
        .await
    }

    #[allow(dead_code)]
    pub async fn load_anchor_account<T: AccountDeserialize>(&mut self, acc_pk: Pubkey) -> T {
        let mut acc = self
            .context
            .banks_client
            .get_account(acc_pk)
            .await
            .unwrap()
            .unwrap();
        let acc_info: AccountInfo = (&acc_pk, &mut acc).into();
        // let data = acc_info.try_borrow_mut_data().unwrap();
        // data.as_slice();
        let buf = &mut acc_info.try_borrow_mut_data().unwrap();
        let buf2 = &mut buf.as_ref();
        T::try_deserialize(buf2).unwrap()
    }

    #[allow(dead_code)]
    pub async fn get_bincode_account<T: serde::de::DeserializeOwned>(
        &mut self,
        address: &Pubkey,
    ) -> T {
        self.context
            .banks_client
            .get_account(*address)
            .await
            .unwrap()
            .map(|a| deserialize::<T>(&a.data).unwrap())
            .unwrap_or_else(|| panic!("GET-TEST-ACCOUNT-ERROR: Account {}", address))
    }

    #[allow(dead_code)]
    pub async fn get_clock(&mut self) -> Clock {
        self.get_bincode_account::<Clock>(&sysvar::clock::id())
            .await
    }

    #[allow(dead_code)]
    pub async fn advance_clock_by_slots(&mut self, slots: u64) {
        let clock: Clock = self.get_clock().await;
        self.context.warp_to_slot(clock.slot + slots).unwrap();
    }

    #[allow(dead_code, unused_variables)]
    pub async fn advance_clock_past_timestamp(&mut self, unix_timestamp: UnixTimestamp) {
        let mut clock: Clock = self.get_clock().await;
        let mut n: i32 = 0;

        while clock.unix_timestamp <= unix_timestamp {
            // Since the exact time is not deterministic keep wrapping by arbitrary 400 slots until we pass the requested timestamp
            self.context.warp_to_slot(clock.slot + 400).unwrap();

            n += 1;
            clock = self.get_clock().await;
        }
    }

    #[allow(dead_code)]
    pub async fn create_token_account_with_amount(
        &mut self,
        owner: &Pubkey,
        mint: &Pubkey,
        amount: u64,
    ) -> Pubkey {
        let keypair = Keypair::new();
        let rent = self.rent.minimum_balance(spl_token::state::Account::LEN) + amount;

        let instructions = [
            system_instruction::create_account(
                &self.context.payer.pubkey(),
                &keypair.pubkey(),
                rent,
                spl_token::state::Account::LEN as u64,
                &spl_token::id(),
            ),
            spl_token::instruction::initialize_account(
                &spl_token::id(),
                &keypair.pubkey(),
                mint,
                owner,
            )
            .unwrap(),
        ];

        self.process_transaction(&instructions, Some(&[&keypair]))
            .await
            .unwrap();
        keypair.pubkey()
    }

    #[allow(dead_code)]
    pub async fn create_token_account(&mut self, owner: &Pubkey, mint: &Pubkey) -> Pubkey {
        self.create_token_account_with_amount(owner, mint, 0).await
    }

    #[allow(dead_code)]
    pub async fn mint_to(
        &mut self,
        authority: &Keypair,
        mint: &Pubkey,
        token_account: &Pubkey,
        num_tokens: u64,
    ) -> Result<(), BanksClientError> {
        let instructions = [spl_token::instruction::mint_to(
            &spl_token::id(),
            mint,
            token_account,
            &authority.pubkey(),
            &[],
            num_tokens,
        )
        .unwrap()];

        self.process_transaction(&instructions, Some(&[authority]))
            .await
    }

    #[allow(dead_code)]
    pub async fn create_and_mint_to_token_account(
        &mut self,
        mint_auth: Option<&Keypair>,
        mint_pubkey: Pubkey,
        payer: &Keypair,
        amount: u64,
        native: bool,
    ) -> Pubkey {
        let auth_copy = Keypair::from_bytes(&self.mint_auth.to_bytes()).unwrap();
        let mint_auth = if let Some(auth) = mint_auth {
            auth
        } else {
            &auth_copy
        };

        if !native {
            let account_pubkey = self
                .create_token_account(&payer.pubkey(), &mint_pubkey)
                .await;

            if amount != 0 {
                self.mint_to(mint_auth, &mint_pubkey, &account_pubkey, amount)
                    .await
                    .unwrap();
            }

            account_pubkey
        } else {
            self.create_token_account_with_amount(&payer.pubkey(), &mint_pubkey, amount)
                .await
        }
    }
}
