use super::*;

use ckb_jsonrpc_types::{IndexerRange, IndexerSearchKeyFilter};
use ckb_types::{
    bytes::Bytes,
    core::{
        capacity_bytes, BlockBuilder, Capacity, EpochNumberWithFraction, HeaderBuilder,
        ScriptHashType, TransactionBuilder,
    },
    packed::{CellInput, CellOutputBuilder, OutPoint, Script, ScriptBuilder},
    H256,
};

#[tokio::test]
async fn test_query_tip() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexerHandle::new(pool.clone(), None);
    let res = indexer.get_indexer_tip().await.unwrap();
    assert!(res.is_none());

    insert_blocks(pool.clone()).await;
    let res = indexer.get_indexer_tip().await.unwrap().unwrap();
    assert_eq!(9, res.block_number.value());
    assert_eq!(
        "953761d56c03bfedf5e70dde0583470383184c41331f709df55d4acab5358640".to_string(),
        res.block_hash.to_string()
    );
}

#[tokio::test]
async fn get_cells() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexerHandle::new(pool.clone(), None);
    let res = indexer.get_indexer_tip().await.unwrap();
    assert!(res.is_none());

    insert_blocks(pool.clone()).await;

    let lock_script = ScriptBuilder::default()
        .code_hash(
            h256!("0x0000000000000000000000000000000000000000000000000000000000000000").pack(),
        )
        .hash_type((ScriptHashType::Data as u8).into())
        .args(
            h160!("0x62e907b15cbf27d5425399ebf6f0fb50ebb88f18")
                .as_bytes()
                .pack(),
        )
        .build();
    let search_key = IndexerSearchKey {
        script: lock_script.into(),
        script_type: IndexerScriptType::Lock,
        script_search_mode: Some(IndexerSearchMode::Prefix),
        filter: Some(IndexerSearchKeyFilter {
            script: None,
            script_len_range: Some(IndexerRange::new(0, 1)),
            output_data: None,
            output_data_filter_mode: None,
            output_data_len_range: Some(IndexerRange::new(0u64, 10u64)),
            output_capacity_range: Some(IndexerRange::new(
                840_000_000_000_000_000_u64,
                840_000_000_100_000_000_u64,
            )),
            block_range: Some(IndexerRange::new(0u64, 10u64)),
        }),
        with_data: Some(false),
        group_by_transaction: None,
    };
    let cells = indexer
        .get_cells(
            search_key,
            IndexerOrder::Asc,
            100u32.into(),
            Some(vec![5u8, 0, 0, 0, 0, 0, 0, 0].pack().into()),
        )
        .await
        .unwrap();

    assert_eq!(cells.objects.len(), 1);
    assert_eq!(
        cells.last_cursor,
        JsonBytes::from_vec(vec![7u8, 0, 0, 0, 0, 0, 0, 0])
    );

    let cell = &cells.objects[0];
    assert_eq!(cell.block_number, 0u64.into());
    assert_eq!(cell.tx_index, 0u32.into());
    assert_eq!(cell.out_point.index, 6u32.into());
    assert_eq!(cell.output.type_, None);
    assert_eq!(cell.output_data, None);

    let type_script = ScriptBuilder::default()
        .code_hash(
            h256!("0x00000000000000000000000000000000000000000000000000545950455f4944").pack(),
        )
        .hash_type((ScriptHashType::Type as u8).into())
        .args(
            h256!("0xb2a8500929d6a1294bf9bf1bf565f549fa4a5f1316a3306ad3d4783e64bcf626")
                .as_bytes()
                .pack(),
        )
        .build();
    let lock_script = ScriptBuilder::default()
        .code_hash(
            h256!("0x0000000000000000000000000000000000000000000000000000000000000000").pack(),
        )
        .hash_type((ScriptHashType::Data as u8).into())
        .args(vec![].as_slice().pack())
        .build();
    let lock_script_len = extract_raw_data(&lock_script).len() as u64;
    let search_key = IndexerSearchKey {
        script: type_script.into(),
        script_type: IndexerScriptType::Type,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: Some(IndexerSearchKeyFilter {
            script: None,
            script_len_range: Some(IndexerRange::new(lock_script_len, lock_script_len + 1)),
            output_data: None,
            output_data_filter_mode: None,
            output_data_len_range: None,
            output_capacity_range: Some(IndexerRange::new(
                16_00_000_000_000_u64,
                16_00_100_000_000_u64,
            )),
            block_range: Some(IndexerRange::new(0u64, 1u64)),
        }),
        with_data: Some(false),
        group_by_transaction: None,
    };
    let cells = indexer
        .get_cells(
            search_key,
            IndexerOrder::Asc,
            10u32.into(),
            Some(vec![1u8, 0, 0, 0, 0, 0, 0, 0].pack().into()),
        )
        .await
        .unwrap();
    assert_eq!(cells.objects.len(), 1);
}

#[tokio::test]
async fn get_cells_filter_data() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexerHandle::new(pool.clone(), None);
    let res = indexer.get_indexer_tip().await.unwrap();
    assert!(res.is_none());

    insert_blocks(pool.clone()).await;

    let search_key = IndexerSearchKey {
        script: ScriptBuilder::default()
            .code_hash(
                h256!("0x00000000000000000000000000000000000000000000000000545950455f4944").pack(),
            )
            .hash_type((ScriptHashType::Type as u8).into())
            .args(
                hex::decode("b2a8500929d6a1294bf9bf1bf565f549fa4a5f1316a3306ad3d4783e64bcf626")
                    .expect("Decoding failed")
                    .pack(),
            )
            .build()
            .into(),
        script_type: IndexerScriptType::Type,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: Some(IndexerSearchKeyFilter {
            script: None,
            script_len_range: None,
            output_data: Some(JsonBytes::from_vec(vec![127, 69, 76])),
            output_data_filter_mode: Some(IndexerSearchMode::Prefix),
            output_data_len_range: None,
            output_capacity_range: None,
            block_range: None,
        }),
        with_data: Some(false),
        group_by_transaction: None,
    };
    let cells = indexer
        .get_cells(
            search_key,
            IndexerOrder::Asc,
            100u32.into(),
            Some(vec![2u8, 0, 0, 0, 0, 0, 0, 0].pack().into()),
        )
        .await
        .unwrap();

    assert_eq!(cells.objects.len(), 1);
    assert_eq!(
        cells.last_cursor,
        JsonBytes::from_vec(vec![3u8, 0, 0, 0, 0, 0, 0, 0])
    );

    let cell = &cells.objects[0];
    assert_eq!(cell.block_number, 0u64.into());
    assert_eq!(cell.tx_index, 0u32.into());
    assert_eq!(cell.out_point.index, 2u32.into());
    assert_eq!(cell.output_data, None);
}

#[tokio::test]
async fn get_cells_by_cursor() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexerHandle::new(pool.clone(), None);
    let res = indexer.get_indexer_tip().await.unwrap();
    assert!(res.is_none());

    insert_blocks(pool.clone()).await;

    let lock_script = ScriptBuilder::default()
        .code_hash(
            h256!("0x0000000000000000000000000000000000000000000000000000000000000000").pack(),
        )
        .hash_type((ScriptHashType::Data as u8).into())
        .args(hex::decode("").expect("Decoding failed").pack())
        .build();
    let search_key = IndexerSearchKey {
        script: lock_script.clone().into(),
        script_type: IndexerScriptType::Lock,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: None,
        with_data: Some(false),
        group_by_transaction: None,
    };
    let first_query_cells = indexer
        .get_cells(
            search_key,
            IndexerOrder::Asc,
            3u32.into(),
            Some(vec![0u8, 0, 0, 0, 0, 0, 0, 0].pack().into()),
        )
        .await
        .unwrap();

    assert_eq!(first_query_cells.objects.len(), 3);
    assert_eq!(
        first_query_cells.last_cursor,
        JsonBytes::from_vec(vec![3u8, 0, 0, 0, 0, 0, 0, 0])
    );

    // query using last_cursor
    let search_key = IndexerSearchKey {
        script: lock_script.into(),
        script_type: IndexerScriptType::Lock,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: None,
        with_data: Some(false),
        group_by_transaction: None,
    };
    let second_query_cells = indexer
        .get_cells(
            search_key,
            IndexerOrder::Asc,
            100u32.into(),
            Some(first_query_cells.last_cursor),
        )
        .await
        .unwrap();

    assert_eq!(second_query_cells.objects.len(), 4);
}

#[tokio::test]
async fn get_transactions() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexerHandle::new(pool.clone(), None);

    insert_blocks(pool).await;

    let lock_script = ScriptBuilder::default()
        .code_hash(
            h256!("0x0000000000000000000000000000000000000000000000000000000000000000").pack(),
        )
        .hash_type((ScriptHashType::Data as u8).into())
        .args(hex::decode("").expect("Decoding failed").pack())
        .build();

    // grouped by transaction
    let search_key = IndexerSearchKey {
        script: lock_script.clone().into(),
        script_type: IndexerScriptType::Lock,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: None,
        with_data: Some(false),
        group_by_transaction: Some(true),
    };
    let txs = indexer
        .get_transactions(search_key, IndexerOrder::Asc, 100u32.into(), None)
        .await
        .unwrap();
    assert_eq!(2, txs.objects.len());

    // ungrouped by transaction
    let search_key = IndexerSearchKey {
        script: lock_script.clone().into(),
        script_type: IndexerScriptType::Lock,
        script_search_mode: Some(IndexerSearchMode::Exact),
        filter: None,
        with_data: Some(false),
        group_by_transaction: None,
    };
    let txs = indexer
        .get_transactions(search_key, IndexerOrder::Asc, 100u32.into(), None)
        .await
        .unwrap();
    assert_eq!(7, txs.objects.len());
}

#[tokio::test]
async fn script_search_mode_rpc() {
    let pool = connect_sqlite(MEMORY_DB).await;
    let indexer = AsyncRichIndexer::new(pool.clone(), None, CustomFilters::new(None, None));
    let rpc = AsyncRichIndexerHandle::new(pool, None);

    // setup test data
    let lock_script1 = ScriptBuilder::default()
        .code_hash(H256(rand::random()).pack())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(b"lock_script1".to_vec()).pack())
        .build();

    let lock_script11 = ScriptBuilder::default()
        .code_hash(lock_script1.code_hash())
        .hash_type(ScriptHashType::Type.into())
        .args(Bytes::from(b"lock_script11".to_vec()).pack())
        .build();

    let type_script1 = ScriptBuilder::default()
        .code_hash(H256(rand::random()).pack())
        .hash_type(ScriptHashType::Data.into())
        .args(Bytes::from(b"type_script1".to_vec()).pack())
        .build();

    let type_script11 = ScriptBuilder::default()
        .code_hash(type_script1.code_hash())
        .hash_type(ScriptHashType::Data.into())
        .args(Bytes::from(b"type_script11".to_vec()).pack())
        .build();

    let cellbase0 = TransactionBuilder::default()
        .input(CellInput::new_cellbase_input(0))
        .witness(Script::default().into_witness())
        .output(
            CellOutputBuilder::default()
                .capacity(capacity_bytes!(1000).pack())
                .lock(lock_script1.clone())
                .build(),
        )
        .output_data(Default::default())
        .build();

    let tx00 = TransactionBuilder::default()
        .output(
            CellOutputBuilder::default()
                .capacity(capacity_bytes!(1000).pack())
                .lock(lock_script1.clone())
                .type_(Some(type_script1.clone()).pack())
                .build(),
        )
        .output_data(Default::default())
        .build();

    let tx01 = TransactionBuilder::default()
        .output(
            CellOutputBuilder::default()
                .capacity(capacity_bytes!(2000).pack())
                .lock(lock_script11.clone())
                .type_(Some(type_script11.clone()).pack())
                .build(),
        )
        .output_data(Default::default())
        .build();

    let block0 = BlockBuilder::default()
        .transaction(cellbase0)
        .transaction(tx00.clone())
        .transaction(tx01.clone())
        .header(HeaderBuilder::default().number(0.pack()).build())
        .build();

    indexer.append(&block0).await.unwrap();

    let (mut pre_tx0, mut pre_tx1, mut pre_block) = (tx00, tx01, block0);
    let total_blocks = 255;
    for i in 1..total_blocks {
        let cellbase = TransactionBuilder::default()
            .input(CellInput::new_cellbase_input(i + 1))
            .witness(Script::default().into_witness())
            .output(
                CellOutputBuilder::default()
                    .capacity(capacity_bytes!(1000).pack())
                    .lock(lock_script1.clone())
                    .build(),
            )
            .output_data(Bytes::from(i.to_string()).pack())
            .build();

        pre_tx0 = TransactionBuilder::default()
            .input(CellInput::new(OutPoint::new(pre_tx0.hash(), 0), 0))
            .output(
                CellOutputBuilder::default()
                    .capacity(capacity_bytes!(1000).pack())
                    .lock(lock_script1.clone())
                    .type_(Some(type_script1.clone()).pack())
                    .build(),
            )
            .output_data(Default::default())
            .build();

        pre_tx1 = TransactionBuilder::default()
            .input(CellInput::new(OutPoint::new(pre_tx1.hash(), 0), 0))
            .output(
                CellOutputBuilder::default()
                    .capacity(capacity_bytes!(2000).pack())
                    .lock(lock_script11.clone())
                    .type_(Some(type_script11.clone()).pack())
                    .build(),
            )
            .output_data(Default::default())
            .build();

        pre_block = BlockBuilder::default()
            .transaction(cellbase)
            .transaction(pre_tx0.clone())
            .transaction(pre_tx1.clone())
            .header(
                HeaderBuilder::default()
                    .number((pre_block.number() + 1).pack())
                    .parent_hash(pre_block.hash())
                    .epoch(
                        EpochNumberWithFraction::new(
                            pre_block.number() + 1,
                            pre_block.number(),
                            1000,
                        )
                        .pack(),
                    )
                    .build(),
            )
            .build();

        indexer.append(&pre_block).await.unwrap();
    }

    // test get_cells rpc with prefix search mode
    let cells = rpc
        .get_cells(
            IndexerSearchKey {
                script: lock_script1.clone().into(),
                ..Default::default()
            },
            IndexerOrder::Asc,
            1000.into(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
            total_blocks as usize + 2,
            cells.objects.len(),
            "total size should be cellbase cells count + 2 (last block live cell: lock_script1 and lock_script11)"
        );

    // test get_cells rpc with exact search mode
    let cells = rpc
        .get_cells(
            IndexerSearchKey {
                script: lock_script1.clone().into(),
                script_search_mode: Some(IndexerSearchMode::Exact),
                ..Default::default()
            },
            IndexerOrder::Asc,
            1000.into(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        total_blocks as usize + 1,
        cells.objects.len(),
        "total size should be cellbase cells count + 1 (last block live cell: lock_script1)"
    );

    // test get_transactions rpc with exact search mode
    let txs = rpc
        .get_transactions(
            IndexerSearchKey {
                script: lock_script1.clone().into(),
                script_search_mode: Some(IndexerSearchMode::Exact),
                ..Default::default()
            },
            IndexerOrder::Asc,
            1000.into(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(total_blocks as usize * 3 - 1, txs.objects.len(), "total size should be cellbase tx count + total_block * 2 - 1 (genesis block only has one tx)");

    // test get_transactions rpc group by tx hash with exact search mode
    let txs = rpc
        .get_transactions(
            IndexerSearchKey {
                script: lock_script1.clone().into(),
                script_search_mode: Some(IndexerSearchMode::Exact),
                group_by_transaction: Some(true),
                ..Default::default()
            },
            IndexerOrder::Asc,
            1000.into(),
            None,
        )
        .await
        .unwrap();

    assert_eq!(
        total_blocks as usize * 2,
        txs.objects.len(),
        "total size should be cellbase tx count + total_block"
    );

    // test get_cells_capacity rpc with exact search mode
    let capacity = rpc
        .get_cells_capacity(IndexerSearchKey {
            script: lock_script1.clone().into(),
            script_search_mode: Some(IndexerSearchMode::Exact),
            ..Default::default()
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        1000 * 100000000 * (total_blocks + 1),
        capacity.capacity.value(),
        "cellbases + last block live cell"
    );

    // test get_cells_capacity rpc with prefix search mode (by default)
    let capacity = rpc
        .get_cells_capacity(IndexerSearchKey {
            script: lock_script1.into(),
            ..Default::default()
        })
        .await
        .unwrap()
        .unwrap();

    assert_eq!(
        1000 * 100000000 * (total_blocks + 1) + 2000 * 100000000,
        capacity.capacity.value()
    );
}

/// helper fn extracts script fields raw data
fn extract_raw_data(script: &Script) -> Vec<u8> {
    [
        script.code_hash().as_slice(),
        script.hash_type().as_slice(),
        &script.args().raw_data(),
    ]
    .concat()
}
