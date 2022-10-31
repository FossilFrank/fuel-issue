use fuels::{
    prelude::*,
    tx::{AssetId, Bytes32, StorageSlot},
};
use std::{str::FromStr, vec};

///////////////////////////////
// Load the Exchange Contract abi
///////////////////////////////
abigen!(TestExchange, "./out/debug/exchange_contract-abi.json");

///////////////////////////////
// Load the Token Contract abi
///////////////////////////////T
abigen!(
    TestToken,
    "../token_contract/out/debug/token_contract-abi.json"
);

struct Fixture {
    wallet: WalletUnlocked,
    #[allow(dead_code)]
    token_contract_id: Bech32ContractId,
    exchange_contract_id: Bech32ContractId,
    token_asset_id: AssetId,
    #[allow(dead_code)]
    exchange_asset_id: AssetId,
    #[allow(dead_code)]
    token_instance: TestToken,
    exchange_instance: TestExchange,
}

fn to_9_decimal(num: u64) -> u64 {
    num * 1_000_000_000
}

async fn setup() -> Fixture {
    let num_wallets = 1;
    let num_coins = 1;
    let amount = to_9_decimal(10000);
    let config = WalletsConfig::new(Some(num_wallets), Some(num_coins), Some(amount));

    let mut wallets = launch_custom_provider_and_get_wallets(config, None).await;
    let wallet = wallets.pop().unwrap();
    // let wallet = launch_provider_and_get_wallet().await;

    //////////////////////////////////////////
    // Setup contracts
    //////////////////////////////////////////

    let token_contract_id = Contract::deploy(
        "../token_contract/out/debug/token_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::new(None, None),
    )
    .await
    .unwrap();

    let key =
        Bytes32::from_str("0x0000000000000000000000000000000000000000000000000000000000000001")
            .unwrap();
    let value = token_contract_id.hash();
    let storage_slot = StorageSlot::new(key, value);
    let storage_vec = vec![storage_slot.clone()];

    // Deploy contract and get ID
    let exchange_contract_id = Contract::deploy(
        "../exchange_contract/out/debug/exchange_contract.bin",
        &wallet,
        TxParameters::default(),
        StorageConfiguration::with_manual_storage(Some(storage_vec)),
    )
    .await
    .unwrap();

    let exchange_instance = TestExchange::new(exchange_contract_id.clone(), wallet.clone());
    let token_instance = TestToken::new(token_contract_id.clone(), wallet.clone());

    let wallet_token_amount = to_9_decimal(20000);

    // Initialize token contract
    token_instance
        .methods()
        .initialize(wallet_token_amount, wallet.address().into())
        .call()
        .await
        .unwrap();

    // Mint some alt tokens
    token_instance
        .methods()
        .mint()
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    Fixture {
        wallet: wallet,
        token_contract_id: token_contract_id.clone(),
        exchange_contract_id: exchange_contract_id.clone(),
        token_asset_id: AssetId::new(*token_contract_id.hash()),
        exchange_asset_id: AssetId::new(*exchange_contract_id.hash()),
        token_instance: token_instance,
        exchange_instance: exchange_instance,
    }
}

async fn add_liquidity(fixture: &Fixture, token_0_amount: u64, token_1_amount: u64) {
    let pool_info = fixture
        .exchange_instance
        .methods()
        .get_pool_info()
        .call()
        .await
        .unwrap();
    println!("{:?}", pool_info.value);

    let _receipts = fixture
        .wallet
        .force_transfer_to_contract(
            &fixture.exchange_contract_id,
            token_0_amount,
            BASE_ASSET_ID,
            TxParameters::default(),
        )
        .await;

    // Deposit some Token Asset
    let _receipts = fixture
        .wallet
        .force_transfer_to_contract(
            &fixture.exchange_contract_id,
            token_1_amount,
            fixture.token_asset_id.clone(),
            TxParameters::default(),
        )
        .await;

    let _result = fixture
        .exchange_instance
        .methods()
        .add_liquidity(Identity::Address(fixture.wallet.address().into()))
        .append_variable_outputs(2)
        .tx_params(TxParameters {
            gas_price: 0,
            gas_limit: 100_000_000,
            maturity: 0,
        })
        .call_params(CallParameters::new(None, None, Some(100_000_000)))
        .call()
        .await
        .unwrap();
}

#[tokio::test]
async fn swap_token_0() {
    let fixture = setup().await;

    let token_0_amount = to_9_decimal(5);
    let token_1_amount = to_9_decimal(10);

    add_liquidity(&fixture, token_0_amount, token_1_amount).await;

    let swap_amount = to_9_decimal(1);
    let expected_output = 1662497915;

    let token_0_starting_balance = fixture
        .wallet
        .get_asset_balance(&BASE_ASSET_ID)
        .await
        .unwrap();
    let token_1_starting_balance = fixture
        .wallet
        .get_asset_balance(&fixture.token_asset_id)
        .await
        .unwrap();

    let _receipt = fixture
        .exchange_instance
        .methods()
        .swap(
            0,
            expected_output,
            Identity::Address(fixture.wallet.address().into()),
        )
        .call_params(CallParameters::new(Some(swap_amount), None, None))
        .tx_params(TxParameters {
            gas_price: 0,
            gas_limit: 100_000_000,
            maturity: 0,
        })
        .append_variable_outputs(1)
        .call()
        .await
        .unwrap();

    let pool_info = fixture
        .exchange_instance
        .methods()
        .get_pool_info()
        .call()
        .await
        .unwrap();
    assert_eq!(
        pool_info.value.token_0_reserve,
        token_0_amount + swap_amount
    );
    assert_eq!(
        pool_info.value.token_1_reserve,
        token_1_amount - expected_output
    );

    let exchange_token_0_balance = fixture
        .wallet
        .get_provider()
        .unwrap()
        .get_contract_asset_balance(&fixture.exchange_contract_id, BASE_ASSET_ID)
        .await
        .unwrap();
    let exchange_token_1_balance = fixture
        .wallet
        .get_provider()
        .unwrap()
        .get_contract_asset_balance(&fixture.exchange_contract_id, fixture.token_asset_id)
        .await
        .unwrap();
    assert_eq!(exchange_token_0_balance, token_0_amount + swap_amount);
    assert_eq!(exchange_token_1_balance, token_1_amount - expected_output);

    let token_0_end_balance = fixture
        .wallet
        .get_asset_balance(&BASE_ASSET_ID)
        .await
        .unwrap();
    let token_1_end_balance = fixture
        .wallet
        .get_asset_balance(&fixture.token_asset_id)
        .await
        .unwrap();

    assert_eq!(token_0_end_balance, token_0_starting_balance - swap_amount);
    assert_eq!(
        token_1_end_balance,
        token_1_starting_balance + expected_output
    );
}
