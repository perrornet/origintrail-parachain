use cumulus_primitives_core::ParaId;
use parachain_runtime::{AccountId, Signature, EVMConfig, EthereumConfig, GLMR, InflationInfo, Range, AuthorFilterConfig};
use sc_chain_spec::{ChainSpecExtension, ChainSpecGroup};
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_core::{H160, U256, Pair, Public, sr25519};
use sp_runtime::{
	traits::{BlakeTwo256, Hash, IdentifyAccount, Verify},
	Perbill,
};

use pallet_evm::GenesisAccount;


use std::collections::BTreeMap;
use std::str::FromStr;
use serde_json as json;
use nimbus_primitives::NimbusId;

use std::convert::TryInto;

const DEFAULT_PROPERTIES_TESTNET: &str = r#"
{
"tokenSymbol": "TTRAC",
"tokenDecimals": 18,
"ss58Format": 42
}
"#;

/// Specialized `ChainSpec` for the normal parachain runtime.
pub type ChainSpec = sc_service::GenericChainSpec<parachain_runtime::GenesisConfig, Extensions>;

/// Helper function to generate a crypto pair from seed
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

/// The extensions for the [`ChainSpec`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, ChainSpecGroup, ChainSpecExtension)]
#[serde(deny_unknown_fields)]
pub struct Extensions {
	/// The relay chain of the Parachain.
	pub relay_chain: String,
	/// The id of the Parachain.
	pub para_id: u32,
}

impl Extensions {
	/// Try to get the extension from the given `ChainSpec`.
	pub fn try_get(chain_spec: &dyn sc_service::ChainSpec) -> Option<&Self> {
		sc_chain_spec::get_extension(chain_spec.extensions())
	}
}

type AccountPublic = <Signature as Verify>::Signer;

/// Helper function to generate an account ID from seed
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

pub fn development_config(para_id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Development",
		// ID
		"dev",
		ChainType::Local,
		move || {
			testnet_genesis(
				AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
				// Validator
				vec![(
					AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
					None,
					1_000 * GLMR,
				)],
				moonbeam_inflation_config(),
				vec![AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap()],
				Default::default(), // para_id
				2160,               //ChainId
			)
		},
		vec![],
		None,
		None,
		Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
		Extensions {
			relay_chain: "rococo-dev".into(),
			para_id: para_id.into(),
		},
	)
}

pub fn local_testnet_config(para_id: ParaId) -> ChainSpec {
	ChainSpec::from_genesis(
		// Name
		"Local Testnet",
		// ID
		"local_testnet",
		ChainType::Local,
		move || {
			testnet_genesis(
				AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
				// Validator
				vec![(
					AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap(),
					None,
					1_000 * GLMR,
				)],
				moonbeam_inflation_config(),
				vec![AccountId::from_str("6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b").unwrap()],
				para_id,
				2160, //ChainId
			)
		},
		vec![],
		None,
		None,
		Some(json::from_str(DEFAULT_PROPERTIES_TESTNET).unwrap()),
		Extensions {
			relay_chain: "rococo-local".into(),
			para_id: para_id.into(),
		},
	)
}


pub fn moonbeam_inflation_config() -> InflationInfo<parachain_runtime::Balance> {
	InflationInfo {
		expect: Range {
			min: 100_000 * GLMR,
			ideal: 200_000 * GLMR,
			max: 500_000 * GLMR,
		},
		annual: Range {
			min: Perbill::from_percent(4),
			ideal: Perbill::from_percent(5),
			max: Perbill::from_percent(5),
		},
		// 8766 rounds (hours) in a year
		round: Range {
			min: Perbill::from_parts(Perbill::from_percent(4).deconstruct() / 8766),
			ideal: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
			max: Perbill::from_parts(Perbill::from_percent(5).deconstruct() / 8766),
		},
	}
}

fn testnet_genesis(
	root_key: AccountId,
	stakers: Vec<(AccountId, Option<AccountId>, parachain_runtime::Balance)>,
	inflation_config: InflationInfo<parachain_runtime::Balance>,
	endowed_accounts: Vec<AccountId>,
	para_id: ParaId,
	chain_id: u64,
) -> parachain_runtime::GenesisConfig {
	let precompile_addresses = vec![1, 2, 3, 4, 5, 6, 7, 8, 1024, 1025, 2048]
		.into_iter()
		.map(H160::from_low_u64_be);

	parachain_runtime::GenesisConfig {
		frame_system: parachain_runtime::SystemConfig {
			code: parachain_runtime::WASM_BINARY
				.expect("WASM binary was not build, please build it!")
				.to_vec(),
			changes_trie_config: Default::default(),
		},
		pallet_balances: parachain_runtime::BalancesConfig {
			balances: endowed_accounts
				.iter()
				.cloned()
				.map(|k| (k, 1 << 60))
				.collect(),
		},
		pallet_sudo: parachain_runtime::SudoConfig { key: root_key },
		parachain_info: parachain_runtime::ParachainInfoConfig { parachain_id: para_id },
		pallet_evm: EVMConfig {
			accounts: precompile_addresses
				.map(|a| {
					(
						a,
						GenesisAccount {
							nonce: Default::default(),
							balance: Default::default(),
							storage: Default::default(),
							code: revert_bytecode.clone(),
						},
					)
				})
				.collect(),

		},
		pallet_ethereum: EthereumConfig {},
		parachain_staking: parachain_runtime::ParachainStakingConfig {
			stakers: stakers.clone(),
			inflation_config,
		},
		pallet_author_slot_filter: parachain_runtime::AuthorFilterConfig { eligible_ratio: 50 },
		pallet_author_mapping: parachain_runtime::AuthorMappingConfig {
			// Pretty hacky. We just set the first staker to use alice's session keys.
			// Maybe this is the moment we should finally make the `--alice` flags make sense.
			// Which is to say, we should prefund the alice account. Actually, I think we already do that...
			author_ids: stakers
				.iter()
				.take(1)
				.map(|staker| {
					let author_id = get_from_seed::<NimbusId>("Alice");
					let account_id = staker.0;
					// This println confirmed that I mapped Alice's session key to Gerald's account ID
					// Now I'm disabling it because it also showed up in my parachain genesis state file
					// println!(
					// 	"Initializing author -> account mapping: ({:?}, {:?})",
					// 	author_id, account_id
					// );
					(author_id, account_id)
				})
				.collect(),
		},
	}
}
