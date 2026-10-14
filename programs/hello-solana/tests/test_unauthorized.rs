
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
fn test_unauthorized_increment() {
    // Wallet A: autoridad legítima.
    let authority = Keypair::new();

    // Wallet B: intentará incrementar sin autorización.
    let attacker = Keypair::new();

    assert_ne!(authority.pubkey(), attacker.pubkey());

    // Identificamos nuestro programa y derivamos la PDA.
    let program_id = hello_solana::id();

    let counter = Pubkey::find_program_address(
        &[hello_solana::constants::COUNTER_SEED],
        &program_id,
    )
    .0;

    // Creamos una red simulada y cargamos el programa.
    let mut svm = LiteSVM::new();

    let bytes = include_bytes!(concat!(
        env!("CARGO_TARGET_TMPDIR"),
        "/../deploy/hello_solana.so"
    ));

    svm.add_program(program_id, bytes).unwrap();

    // Entregamos SOL simulado a la autoridad.
    svm.airdrop(&authority.pubkey(), 1_000_000_000)
        .unwrap();

    // Construimos la instrucción initialize.
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

    // Creamos y firmamos la transacción con Wallet A.
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

    // La inicialización debe funcionar.
    let result = svm.send_transaction(tx);
    assert!(result.is_ok());

    // Verificamos que la Wallet A sea la autoridad.
    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;

    let counter_state =
        hello_solana::state::Counter::try_deserialize(&mut data)
            .unwrap();

    assert_eq!(counter_state.count, 0);
    assert_eq!(counter_state.authority, authority.pubkey());


    // Financiamos la Wallet B para pagar la comisión de prueba.
    svm.airdrop(&attacker.pubkey(), 1_000_000_000)
        .unwrap();

    // Wallet B intenta incrementar el contador.
    let instruction = Instruction::new_with_bytes(
        program_id,
        &hello_solana::instruction::Increment {}.data(),
        hello_solana::accounts::Increment {
            counter,
            authority: attacker.pubkey(),
        }
        .to_account_metas(None),
    );

    // La transacción es firmada por la Wallet B.
    let blockhash = svm.latest_blockhash();

    let msg = Message::new_with_blockhash(
        &[instruction],
        Some(&attacker.pubkey()),
        &blockhash,
    );

    let tx = VersionedTransaction::try_new(
        VersionedMessage::Legacy(msg),
        &[&attacker],
    )
    .unwrap();

    // Esperamos que el programa rechace la operación.

    // El intento debe fallar específicamente con Unauthorized (6000).
    let result = svm.send_transaction(tx);

    let error = result.expect_err(
        "Una wallet no autorizada logró incrementar el contador"
    );

    assert!(
        format!("{error:?}").contains("InstructionError(0, Custom(6000))"),
        "Se esperaba Unauthorized (6000), pero ocurrió: {error:?}"
    );

    // Comprobamos que el contador no haya cambiado.
    let counter_account = svm.get_account(&counter).unwrap();
    let mut data: &[u8] = &counter_account.data;

    let counter_state =
        hello_solana::state::Counter::try_deserialize(&mut data)
            .unwrap();

    assert_eq!(counter_state.count, 0);
    assert_eq!(counter_state.authority, authority.pubkey());

}
