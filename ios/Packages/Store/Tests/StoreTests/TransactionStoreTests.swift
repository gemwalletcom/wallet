// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GRDB
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct TransactionStoreTests {
    @Test func transactionWalletIncludesAccounts() throws {
        let asset = AssetBasic.mock(asset: .mock(id: Chain.robinhood.assetId))
        let db = DB.mockAssets(assets: [asset])
        let walletStore = WalletStore(db: db)
        let walletId = WalletId.single(chain: .robinhood, address: "0xsender")
        let account = Account.mock(chain: .robinhood, address: "0xsender")
        try walletStore.addWallet(.mock(id: walletId, type: .single, accounts: [account]))

        let store = TransactionStore(db: db)
        let transactionId = TransactionId(chain: .robinhood, hash: "hash")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, assetId: Chain.robinhood.assetId),
        ])

        let transactionWallet = try #require(try store.getTransactionWallet(walletId: walletId, transactionId: transactionId))

        #expect(transactionWallet.wallet.accounts == [account])
    }

    @Test func assetAssociationsReplaced() throws {
        let btc = AssetId(chain: .bitcoin, tokenId: nil)
        let eth = AssetId(chain: .ethereum, tokenId: nil)
        let sol = AssetId(chain: .solana, tokenId: nil)

        let assets: [AssetBasic] = [
            .mock(asset: .mock(id: btc)),
            .mock(asset: .mock(id: eth)),
            .mock(asset: .mock(id: sol)),
        ]

        let db = DB.mockAssets(assets: assets)
        let walletStore = WalletStore(db: db)
        let walletId = WalletId.multicoin(address: "test")
        try walletStore.addWallet(.mock(id: walletId, accounts: assets.map { Account.mock(chain: $0.asset.chain) }))

        let store = TransactionStore(db: db)
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(
                transactionId: transactionId,
                type: .swap,
                assetId: btc,
                metadata: .encode(TransactionSwapMetadata(
                    fromAsset: btc, fromValue: "100", toAsset: eth, toValue: "200", provider: nil,
                )),
            ),
        ])

        try store.addTransactions(walletId: walletId, transactions: [
            .mock(
                transactionId: transactionId,
                type: .swap,
                assetId: btc,
                metadata: .encode(TransactionSwapMetadata(
                    fromAsset: btc, fromValue: "100", toAsset: sol, toValue: "300", provider: nil,
                )),
            ),
        ])

        let assetIds = try store.getTransactionAssetAssociations(for: transactionId).map(\.assetId)

        #expect(assetIds.count == 2)
        #expect(Set(assetIds) == Set([btc, sol]))
    }

    @Test func mergePreservesIdentityAndAssociationsWithinWallet() throws {
        let ethereum = Chain.ethereum.assetId
        let bitcoin = Chain.bitcoin.assetId
        let solana = Chain.solana.assetId
        let db = DB.mockAssets(assets: [ethereum, bitcoin, solana].map { .mock(asset: .mock(id: $0)) })
        let store = TransactionStore(db: db)
        let walletId = WalletId.mock()
        let otherWalletId = WalletId.multicoin(address: "other")
        try WalletStore(db: db).addWallet(.mock(id: otherWalletId))
        let sourceId = TransactionId(chain: .ethereum, hash: "pending")
        let targetId = TransactionId(chain: .ethereum, hash: "confirmed")
        let source = Transaction.mock(
            transactionId: sourceId,
            type: .swap,
            state: .pending,
            assetId: ethereum,
            metadata: .encode(TransactionSwapMetadata(fromAsset: ethereum, fromValue: "100", toAsset: bitcoin, toValue: "200", provider: nil)),
        )
        let target = Transaction.mock(
            transactionId: targetId,
            type: .swap,
            assetId: ethereum,
            metadata: .encode(TransactionSwapMetadata(fromAsset: ethereum, fromValue: "100", toAsset: solana, toValue: "300", provider: nil)),
            fee: "42",
        )
        try store.addTransactions(walletId: walletId, transactions: [source, target])
        try store.addTransactions(walletId: otherWalletId, transactions: [source, target])
        let storedSource = try store.getTransaction(walletId: walletId, transactionId: sourceId)
        let storedTarget = try store.getTransaction(walletId: walletId, transactionId: targetId)
        let sourceRecord = try #require(try db.dbQueue.read {
            try TransactionRecord.filter(TransactionRecord.Columns.walletId == walletId.id)
                .filter(TransactionRecord.Columns.transactionId == sourceId.identifier).fetchOne($0)
        })

        try store.updateTransactionHash(walletId: walletId, transactionId: sourceId, hash: targetId.hash)
        try store.updateTransactionHash(walletId: walletId, transactionId: sourceId, hash: targetId.hash)
        try store.updateTransactionHash(walletId: walletId, transactionId: targetId, hash: targetId.hash)

        let result = try store.getTransaction(walletId: walletId, transactionId: targetId)
        #expect(result.recordId == storedSource.recordId)
        #expect(throws: RecordError.self) {
            try db.dbQueue.read { try TransactionRequest(walletId: otherWalletId, recordId: result.recordId).fetch($0) }
        }
        #expect(result.transaction == storedTarget.transaction)
        #expect(Set(result.assets.map(\.id)) == Set([ethereum, solana]))
        #expect(try store.getTransaction(walletId: otherWalletId, transactionId: sourceId).transaction == storedSource.transaction)
        #expect(try store.getTransaction(walletId: otherWalletId, transactionId: targetId).transaction == storedTarget.transaction)
        try db.dbQueue.read { db in
            let record = try TransactionRecord.filter(TransactionRecord.Columns.walletId == walletId.id).fetchOne(db)
            let count = try TransactionRecord.fetchCount(db)
            #expect(record?.id == sourceRecord.id)
            #expect(count == 3)
        }
    }
}
