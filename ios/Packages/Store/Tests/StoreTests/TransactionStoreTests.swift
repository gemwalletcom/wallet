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

    @Test func syncKeepsIntentAndUpdatesObservedValues() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .swap, state: .inTransit, metadata: Self.swapMetadata),
        ])

        try store.syncTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .transfer, state: .confirmed, metadata: nil, fee: "500", blockNumber: "77"),
        ])

        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction

        #expect(transaction.type == .swap)
        #expect(transaction.state == .confirmed)
        #expect(transaction.metadata?.decode(TransactionSwapMetadata.self)?.provider == "nearintents")
        #expect(transaction.fee == "500")
        #expect(transaction.blockNumber == "77")
    }

    @Test func syncEnrichesStoredTransactionWithIndexedDescription() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .smartContractCall, state: .confirmed, metadata: nil),
        ])

        try store.syncTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .swap, state: .confirmed, metadata: Self.swapMetadata),
        ])

        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction
        let assetIds = try store.getTransactionAssetAssociations(for: transactionId).map(\.assetId)

        #expect(transaction.type == .swap)
        #expect(transaction.metadata?.decode(TransactionSwapMetadata.self)?.provider == "nearintents")
        #expect(Set(assetIds) == Set([Chain.bitcoin.assetId, Chain.ethereum.assetId]))
    }

    @Test func syncSettlesPendingTransaction() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .transfer, state: .pending, metadata: nil),
        ])

        try store.syncTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .transfer, state: .confirmed, metadata: nil),
        ])

        #expect(try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction.state == .confirmed)
    }

    @Test func syncKeepsDescriptionOfTrackedTransaction() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .swap, state: .inTransit, metadata: Self.swapMetadata),
        ])

        try store.syncTransactions(walletId: walletId, transactions: [
            .mock(transactionId: transactionId, type: .swap, state: .inTransit, metadata: Self.indexedSwapMetadata),
        ])

        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction

        #expect(transaction.metadata?.decode(TransactionSwapMetadata.self)?.provider == "nearintents")
    }

    @Test func syncStoresUnknownTransactionOnce() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        let indexed = Transaction.mock(transactionId: transactionId, type: .swap, state: .confirmed, metadata: Self.swapMetadata)

        try store.syncTransactions(walletId: walletId, transactions: [indexed])
        try store.syncTransactions(walletId: walletId, transactions: [indexed])

        let transaction = try store.getTransaction(walletId: walletId, transactionId: transactionId).transaction

        #expect(try store.getTransactions(states: TransactionState.allCases).count == 1)
        #expect(transaction.type == .swap)
        #expect(transaction.state == .confirmed)
        #expect(transaction.metadata?.decode(TransactionSwapMetadata.self)?.provider == "nearintents")
    }

    @Test func syncStoresRepeatedBatchOnce() throws {
        let (store, walletId) = transactionStore()
        let transactionId = TransactionId(chain: .ethereum, hash: "1")
        let indexed = Transaction.mock(transactionId: transactionId, type: .transfer, state: .confirmed, metadata: nil)

        try store.syncTransactions(walletId: walletId, transactions: [indexed, indexed])

        #expect(try store.getTransactions(states: TransactionState.allCases).count == 1)
    }

    @Test func hashChangeKeepsTrackedTransactionOverIndexedDuplicate() throws {
        let (store, walletId) = transactionStore()
        let localId = TransactionId(chain: .ethereum, hash: "message")
        let indexedId = TransactionId(chain: .ethereum, hash: "onchain")
        try store.addTransactions(walletId: walletId, transactions: [
            .mock(transactionId: localId, type: .swap, state: .inTransit, metadata: Self.swapMetadata),
        ])
        try store.syncTransactions(walletId: walletId, transactions: [
            .mock(transactionId: indexedId, type: .transfer, state: .confirmed, metadata: nil),
        ])

        try store.updateTransactionId(oldTransactionId: localId, transactionId: indexedId, hash: indexedId.hash)
        let transaction = try store.getTransaction(walletId: walletId, transactionId: indexedId).transaction

        #expect(try store.getTransactions(states: TransactionState.allCases).count == 1)
        #expect(transaction.type == .swap)
        #expect(transaction.state == .inTransit)
        #expect(transaction.metadata?.decode(TransactionSwapMetadata.self)?.provider == "nearintents")
    }
}

extension TransactionStoreTests {
    private static let swapMetadata = AnyCodableValue.encode(TransactionSwapMetadata.mock(
        fromAsset: Chain.bitcoin.assetId,
        toAsset: Chain.ethereum.assetId,
        provider: "nearintents",
    ))

    private static let indexedSwapMetadata = AnyCodableValue.encode(TransactionSwapMetadata.mock(
        fromAsset: Chain.bitcoin.assetId,
        toAsset: Chain.ethereum.assetId,
        provider: nil,
    ))

    private func transactionStore() -> (TransactionStore, WalletId) {
        let db = DB.mockAssets(assets: [
            .mock(asset: .mock(id: Chain.bitcoin.assetId)),
            .mock(asset: .mockEthereum()),
        ])
        return (.mock(db: db), .mock())
    }
}
