// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemFiatServiceProtocol
import GemstonePrimitives
import Primitives

public protocol FiatQuoting: Sendable {
    func getQuotes(walletId: WalletId, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote]
    func getQuoteUrl(walletId: WalletId, quoteId: String) async throws -> FiatQuoteUrl
}

public struct FiatService: Sendable {
    private let apiService: any GemFiatServiceProtocol

    public init(apiService: any GemFiatServiceProtocol) {
        self.apiService = apiService
    }

    public func updateTransactions(walletId: WalletId) async throws {
        try await apiService.syncTransactions(walletId: walletId.id)
    }
}

extension FiatService: FiatQuoting {
    public func getQuotes(walletId: WalletId, type: FiatQuoteType, assetId: AssetId, request: FiatQuoteRequest) async throws -> [FiatQuote] {
        try await apiService.getQuotes(
            walletId: walletId.id,
            quoteType: type.json(),
            assetId: assetId.identifier,
            amount: request.amount,
            currency: try Currency(id: request.currency).json(),
        ).map { try FiatQuote($0) }
    }

    public func getQuoteUrl(walletId: WalletId, quoteId: String) async throws -> FiatQuoteUrl {
        try await FiatQuoteUrl(apiService.getQuoteUrl(walletId: walletId.id, quoteId: quoteId))
    }
}
