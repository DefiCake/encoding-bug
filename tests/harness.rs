
mod tests {
    use std::mem::size_of;

    use fuels::{
        accounts::{fuel_crypto::SecretKey, wallet::WalletUnlocked}, 
        prelude::{abigen, AssetId, setup_test_provider, Contract, TxParameters, CallParameters}, 
        test_helpers::{AssetConfig, setup_custom_assets_coins, Config}
    };

    abigen!(
        Contract(
            name = "MyContract",
            abi = "/Users/mad/fuel/cajon/test-bad-shit/my-fuel-project/out/debug/my-fuel-project-abi.json",
        )
    );

    const CONTRACT_BINARY: &str = "out/debug/my-fuel-project.bin";

    pub const DEFAULT_COIN_AMOUNT: u64 = 1_000_000_000;

    fn create_wallet() -> WalletUnlocked {
        const SIZE_SECRET_KEY: usize = size_of::<SecretKey>();
        const PADDING_BYTES: usize = SIZE_SECRET_KEY - size_of::<u64>();
        let mut secret_key: [u8; SIZE_SECRET_KEY] = [0; SIZE_SECRET_KEY];
        secret_key[PADDING_BYTES..].copy_from_slice(&(8320147306839812359u64).to_be_bytes());
    
        let wallet = WalletUnlocked::new_from_private_key(
            SecretKey::try_from(secret_key.as_slice()).unwrap(),
            None,
        );
        wallet
    }

    #[tokio::test]
    async fn test() {
        let mut wallet = create_wallet();
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Generate coins for wallet
        let asset_configs = vec![AssetConfig {
                id: coin.1,
                num_coins: 1,
                coin_amount: coin.0,
        }];

        let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);
        let provider = setup_test_provider(
            all_coins.clone(),
            vec![],
            Some(Config::local_node()),
            None,
        )
        .await;

        wallet.set_provider(provider);

        let test_contract_id =
            Contract::load_from(CONTRACT_BINARY, Default::default())
                .unwrap()
                .deploy(&wallet.clone(), Default::default())
                .await
                .unwrap();

        let contract = MyContract::new(test_contract_id.clone(), wallet.clone());
        let gas = 10_000;

        let call_response = contract
            .methods()
            .test_function(42)
            .tx_params(TxParameters::new(Some(0), Some(gas), 0))
            .call_params(CallParameters::new(0, Default::default(), gas))
            .expect("Call param Error")
            .call()
            .await
            .unwrap();

        dbg!(call_response.value);
        dbg!(call_response.receipts);
    
        assert_eq!(42, call_response.value);
    }
}