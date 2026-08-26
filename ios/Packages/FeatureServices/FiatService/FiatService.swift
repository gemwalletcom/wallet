// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import Foundation
import protocol Gemstone.GemFiatServiceProtocol
import GemstonePrimitives
import Primitives
import Store

public protocol FiatQuoting: Sendable {
    func getQuotes(walletId: WalletId, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote]
    func getQuoteUrl(walletId: WalletId, quoteId: String) async throws -> FiatQuoteUrl
}

public struct FiatService: Sendable {
    private let apiService: any GemFiatServiceProtocol
    private let assetsService: AssetsService
    private let store: FiatTransactionStore

    public init(
        apiService: any GemFiatServiceProtocol,
        assetsService: AssetsService,
        store: FiatTransactionStore,
    ) {
        self.apiService = apiService
        self.assetsService = assetsService
        self.store = store
    }

    public func updateTransactions(walletId: WalletId) async throws {
        let transactions = try await getFiatTransactions(walletId: walletId)
        try await prefetchAssets(transactions: transactions)
        try store.addTransactions(walletId: walletId, transactions: transactions)
    }

    public func getFiatTransactions(walletId: WalletId) async throws -> [FiatTransactionData] {
        try await apiService.getTransactions(walletId: walletId.id).map { try FiatTransactionData($0) }
    }
}

extension FiatService: FiatQuoting {
    public func getQuotes(walletId: WalletId, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote] {
        try await apiService.getQuotes(
            walletId: walletId.id,
            quoteType: type.json(),
            assetId: assetId.identifier,
            amount: request.amount,
            currency: request.currency,
        ).map { try FiatQuote($0) }
    }

    public func getQuoteUrl(walletId: WalletId, quoteId: String) async throws -> FiatQuoteUrl {
        try await FiatQuoteUrl(apiService.getQuoteUrl(walletId: walletId.id, quoteId: quoteId))
    }
}

extension FiatService {
    private func prefetchAssets(transactions: [FiatTransactionData]) async throws {
        try await assetsService.prefetchAssets(assetIds: transactions.map(\.transaction.assetId))
    }
}
