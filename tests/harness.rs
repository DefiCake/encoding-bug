mod tests {
    const CONTRACT_MESSAGE_SCRIPT_BINARY: &str = "bin/contract_message_script.bin";
    const CONTRACT_MESSAGE_PREDICATE_BINARY: &str = "bin/contract_message_predicate.bin";

    use std::{mem::size_of, str::FromStr};

    use fuel_core_types::fuel_types::Nonce;
    use fuel_tx::{Address, Bytes32, Output, TxPointer, UtxoId};
    use fuels::{
        accounts::{fuel_crypto::SecretKey, predicate::Predicate, wallet::WalletUnlocked, Signer},
        prelude::{abigen, setup_test_provider, AssetId, Contract, TxPolicies},
        test_helpers::{setup_custom_assets_coins, setup_single_message, AssetConfig},
        types::{
            bech32::Bech32Address,
            coin_type::CoinType,
            input::Input,
            transaction::ScriptTransaction,
            transaction_builders::{
                BuildableTransaction, ScriptTransactionBuilder, TransactionBuilder,
            },
        },
    };

    abigen!(Contract(
        name = "MyContract",
        abi = "out/debug/my-fuel-project-abi.json",
    ));

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
    async fn test_relay_message() {
        let mut wallet = create_wallet();
        let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

        // Generate coins for wallet
        let asset_configs = vec![AssetConfig {
            id: coin.1,
            num_coins: 1,
            coin_amount: coin.0,
        }];

        let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);
        // let predicate = Predicate::load_from(CONTRACT_MESSAGE_PREDICATE_BINARY).unwrap();
        let predicate = Predicate::load_from(CONTRACT_MESSAGE_PREDICATE_BINARY).unwrap();
        let predicate_root = predicate.address();

        let message = {
            let sender: Bech32Address = Address::from_str(
                "0x00000000000000000000000096c53cd98B7297564716a8f2E1de2C83928Af2fe",
            )
            .unwrap()
            .into();
            let recipient = predicate_root;
            let nonce = Nonce::zeroed();
            let amount = 0u64;
            let data = [].to_vec();

            setup_single_message(&sender, &recipient, amount, nonce, data)
        };
        let provider = setup_test_provider(all_coins.clone(), vec![message.clone()], None, None)
            .await
            .expect("Could not instantiate provider");

        wallet.set_provider(provider.clone());

        let test_contract_id = Contract::load_from(CONTRACT_BINARY, Default::default())
            .unwrap()
            .deploy(&wallet.clone(), Default::default())
            .await
            .unwrap();

        dbg!(&test_contract_id);

        // Prove that message exists
        let messages = &provider.get_messages(&message.recipient).await.unwrap();
        let message = messages.first().unwrap().to_owned();

        let tx: ScriptTransaction = {
            let network_info = &provider.network_info().await.unwrap();

            let predicate = Predicate::load_from(CONTRACT_MESSAGE_PREDICATE_BINARY)
                .unwrap()
                .with_provider(provider.clone());

            let message_input = Input::resource_predicate(
                CoinType::Message(message),
                predicate.code().clone(),
                Default::default(),
            );

            let contract_input = Input::contract(
                UtxoId::new(Bytes32::zeroed(), 0u8),
                Bytes32::zeroed(),
                Bytes32::zeroed(),
                TxPointer::default(),
                test_contract_id.into(),
            );

            let coins: Vec<Input> = provider
                .get_coins(wallet.address(), Default::default())
                .await
                .unwrap()
                .iter()
                .map(|el| {
                    Input::resource_signed(fuels::types::coin_type::CoinType::Coin(el.clone()))
                })
                .collect();

            let coin_input = coins.first().unwrap().clone();

            let contract_output = Output::contract(1u8, Bytes32::zeroed(), Bytes32::zeroed());
            let script_bytecode = std::fs::read(CONTRACT_MESSAGE_SCRIPT_BINARY).unwrap();

            let mut builder = ScriptTransactionBuilder::new(network_info.to_owned())
                .with_inputs(vec![
                    message_input,  // The message to relay
                    contract_input, // The contract where the message is going to be sent
                    coin_input,     // The coins to fund the transaction
                ])
                .with_outputs(vec![contract_output])
                .with_script(script_bytecode)
                .with_tx_policies(TxPolicies::new(Some(0), None, Some(0), None, Some(30_000)));

            wallet.sign_transaction(&mut builder);
            builder.build(provider.clone()).await.unwrap()
        };

        let tx = &provider
            .send_transaction(tx)
            .await
            .expect("Transaction failed");

        let receipts = &provider
            .tx_status(tx)
            .await
            .expect("Failed to fetch transaction status")
            .take_receipts();

        dbg!(receipts);

        panic!("Fail to dbg receipt");
    }

    // #[tokio::test]
    // async fn test_function() {
    //     let mut wallet = create_wallet();
    //     let coin = (DEFAULT_COIN_AMOUNT, AssetId::default());

    //     // Generate coins for wallet
    //     let asset_configs = vec![AssetConfig {
    //             id: coin.1,
    //             num_coins: 1,
    //             coin_amount: coin.0,
    //     }];

    //     let all_coins = setup_custom_assets_coins(wallet.address(), &asset_configs[..]);
    //     let predicate = Predicate::load_from(CONTRACT_MESSAGE_PREDICATE_BINARY).unwrap();
    //     let predicate_root = predicate.address();
    //     let message = {
    //         let sender: Bech32Address = Address::from_str("0x00000000000000000000000096c53cd98B7297564716a8f2E1de2C83928Af2fe").unwrap().into();
    //         let recipient = predicate_root;
    //         let nonce = Nonce::zeroed();
    //         let amount = 0u64;
    //         let data = [].to_vec();

    //         setup_single_message(&sender, &recipient, amount, nonce, data)
    //     };
    //     let provider = setup_test_provider(
    //         all_coins.clone(),
    //         vec![message.clone()],
    //         None,
    //         None,
    //     )
    //     .await
    //     .expect("Could not instantiate provider");

    //     wallet.set_provider(provider.clone());

    //     let test_contract_id =
    //         Contract::load_from(CONTRACT_BINARY, Default::default())
    //             .unwrap()
    //             .deploy(&wallet.clone(), Default::default())
    //             .await
    //             .unwrap();

    //     // Prove that message exists
    //     let messages = &provider.get_messages(&message.recipient).await.unwrap();
    //     let message = messages.first().unwrap();

    //     let contract = MyContract::new(test_contract_id.clone(), wallet.clone());
    //     let gas = 10_000;

    //     let call_response = contract
    //         .methods()
    //         .test_function(0)
    //         .with_tx_policies(TxPolicies::new(Some(0), None, Some(0), None, Some(gas)))
    //         .call_params(CallParameters::new(0, Default::default(), gas))
    //         .expect("Call param Error")
    //         .call()
    //         .await
    //         .unwrap();

    //     dbg!(call_response.value);
    //     dbg!(call_response.receipts);
    // }
}
