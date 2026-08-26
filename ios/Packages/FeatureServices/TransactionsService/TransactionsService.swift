// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import protocol Gemstone.GemTransactionsServiceProtocol
import GemstonePrimitives
import Preferences
import Primitives
import Store

public final class TransactionsService: Sendable {
    let provider: any GemTransactionsServiceProtocol
    public let transactionStore: TransactionStore
    let assetsService: AssetsService
    private let addressStore: AddressStore

    public init(
        provider: any GemTransactionsServiceProtocol,
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

        let response = try await TransactionsResponse(provider.getTransactions(
            walletId: walletId.id,
            assetId: nil,
            fromTimestamp: UInt64(store.transactionsTimestamp),
        ))

        try await prefetchAssets(walletId: walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: walletId, transactions: response.transactions)
        try addressStore.updateAddressNames(response.addressNames)

        store.transactionsTimestamp = newTimestamp
    }

    public func updateForAsset(walletId: WalletId, assetId: AssetId) async throws {
        let store = WalletPreferences(walletId: walletId)
        let newTimestamp = Int(Date.now.timeIntervalSince1970)
        let response = try await TransactionsResponse(provider.getTransactions(
            walletId: walletId.id,
            assetId: assetId.identifier,
            fromTimestamp: UInt64(store.transactionsForAssetTimestamp(assetId: assetId.identifier)),
        ))

        try await prefetchAssets(walletId: walletId, transactions: response.transactions)
        try transactionStore.addTransactions(walletId: walletId, transactions: response.transactions)
        try addressStore.updateAddressNames(response.addressNames)

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
