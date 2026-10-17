
use {
    anchor_lang::{
        prelude::Pubkey,
        solana_program::{instruction::Instruction, system_program},
        AccountDeserialize, InstructionData, ToAccountMetas,
    },
    litesvm::LiteSVM,
    solana_keypair::Keypair,
    solana_message::{Message, VersionedMessage},
    solana_signer::Signer,
    solana_transaction::versioned::VersionedTransaction,
};

#[test]
fn test_counter_overflow() {
    // Identificamos el programa y creamos la wallet autorizada.
    let program_id = hello_solana::id();
    let authority = Keypair::new();

    // Derivamos la dirección PDA del contador.
    let counter = Pubkey::find_program_address(
        &[hello_solana::constants::COUNTER_SEED],
        &program_id,
    )
    .0;

    // Preparamos LiteSVM y cargamos nuestro programa compilado.
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/hello_solana.so"
    ));

    svm.add_program(program_id, bytes).unwrap();

    // SOL simulado para crear la cuenta y pagar comisiones.
    svm.airdrop(&authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Construimos la instrucción de inicialización.
    let instruction = Instruction::new_with_bytes(
        program_id,
        &hello_solana::instruction::Initialize {}.data(),
        hello_solana::accounts::Initialize {
            payer: authority.pubkey(),
            counter,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&authority.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&authority],
    )
    .unwrap();

    // La cuenta debe inicializarse correctamente.
    let result = svm.send_transaction(tx);
    assert!(result.is_ok(), "Falló la inicialización: {result:?}");

    // Comprobamos el estado inicial.
    let account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &account.data;

    let state =
        hello_solana::state::Counter::try_deserialize(&mut data)
            .unwrap();

    assert_eq!(state.count, 0);
    assert_eq!(state.authority, authority.pubkey());


    // Ejecutamos diez incrementos autorizados.
    for expected_count in 1..=10 {
        // Usamos un blockhash nuevo para evitar transacciones duplicadas.
        svm.expire_blockhash();

        let instruction = Instruction::new_with_bytes(
            program_id,
            &hello_solana::instruction::Increment {}.data(),
            hello_solana::accounts::Increment {
                counter,
                authority: authority.pubkey(),
            }
            .to_account_metas(None),
        );

        let blockhash = svm.latest_blockhash();

        let msg = Message::new_with_blockhash(
            &[instruction],
            Some(&authority.pubkey()),
            &blockhash,
        );

        let tx = VersionedTransaction::try_new(
            VersionedMessage::Legacy(msg),
            &[&authority],
        )
        .unwrap();

        let result = svm.send_transaction(tx);

        assert!(
            result.is_ok(),
            "Falló el incremento {expected_count}: {result:?}"
        );

        // Verificamos el estado después de cada incremento.
        let account = svm.get_account(&counter).unwrap();
        let mut data: &[u8] = &account.data;

        let state =
            hello_solana::state::Counter::try_deserialize(&mut data)
                .unwrap();

        assert_eq!(state.count, expected_count);

        println!("Incremento {expected_count}/10: OK");
    }


    // Intentamos superar el límite de 10.
    svm.expire_blockhash();

    let instruction = Instruction::new_with_bytes(
        program_id,
        &hello_solana::instruction::Increment {}.data(),
        hello_solana::accounts::Increment {
            counter,
            authority: authority.pubkey(),
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&authority.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&authority],
    )
    .unwrap();

    let result = svm.send_transaction(tx);

    // El error esperado es CounterOverflow (6001).
    let error = result.expect_err(
        "El programa permitió superar el límite de 10"
    );

    assert!(
        format!("{error:?}").contains("InstructionError(0, Custom(6001))"),
        "Se esperaba CounterOverflow (6001), pero ocurrió: {error:?}"
    );

    // Comprobamos que el intento fallido no modificó la cuenta.
    let account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &account.data;

    let state =
        hello_solana::state::Counter::try_deserialize(&mut data)
            .unwrap();

    assert_eq!(state.count, 10);
    assert_eq!(state.authority, authority.pubkey());

    println!("Incremento 11: rechazado con CounterOverflow (6001)");
    println!("Estado final del contador: {}", state.count);


}
