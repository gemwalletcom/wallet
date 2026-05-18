// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import GemAPI
import Preferences
import Primitives
import Store

public final class TransactionsService: Sendable {
    let provider: any GemAPITransactionService
    public let transactionStore: TransactionStore
    let assetsService: AssetsService
    private let addressStore: AddressStore

    public init(
        provider: any GemAPITransactionService,
        transactionStore: TransactionStore,
        assetsService: AssetsService,
        addressStore: AddressStore,
    ) {
        self.provider = provider
        self.transactionStore = transactionStore
        self.assetsService = assetsService
        self.addressStore = addressStore
    }

    public func updateAll(walletId: WalletId) async throws {
        let store = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)

        let response = try await provider.getDeviceTransactions(
            walletId: walletId,
            fromTimestamp: store.transactionsTimestamp,
        )

        try await prefetchAssets(walletId: walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: walletId, transactions: response.transactions)
        try addressStore.addAddressNames(response.addressNames)

        store.transactionsTimestamp = newTimestamp
    }

    public func updateForAsset(walletId: WalletId, assetId: AssetId) async throws {
        let store = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)
        let response = try await provider.getDeviceTransactionsForAsset(
            walletId: walletId,
            asset: assetId,
            fromTimestamp: store.transactionsForAssetTimestamp(assetId: assetId.identifier),
        )
        if response.transactions.isEmpty {
            return
        }

        try await prefetchAssets(walletId: walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: walletId, transactions: response.transactions)
        try addressStore.addAddressNames(response.addressNames)

        store.setTransactionsForAssetTimestamp(assetId: assetId.identifier, value: newTimestamp)
    }

    public func addTransaction(walletId: WalletId, transaction: Transaction) throws {
        try transactionStore.addTransactions(walletId: walletId, transactions: [transaction])
    }

    public func getTransaction(walletId: WalletId, transactionId: TransactionId) throws -> TransactionExtended {
        try transactionStore.getTransaction(walletId: walletId, transactionId: transactionId)
    }

    private func prefetchAssets(walletId: WalletId, transactions: [Transaction]) async throws {
        let assetIds = transactions.map(\.assetIds).flatMap(\.self)
        if assetIds.isEmpty {
            return
        }
        let newAssets = try await assetsService.prefetchAssets(assetIds: assetIds)
        try assetsService.addBalancesIfMissing(walletId: walletId, assetIds: newAssets)
    }
}
