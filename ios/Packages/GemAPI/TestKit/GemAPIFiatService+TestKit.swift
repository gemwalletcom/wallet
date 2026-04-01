// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemAPI
import Primitives

public struct GemAPIFiatServiceMock: GemAPIFiatService {
    private let quotes: [FiatQuote]
    private let quoteUrl: FiatQuoteUrl
    private let fiatTransactions: [FiatTransactionInfo]

    public init(
        quotes: [FiatQuote] = [],
        quoteUrl: FiatQuoteUrl = FiatQuoteUrl(redirectUrl: ""),
        fiatTransactions: [FiatTransactionInfo] = [],
    ) {
        self.quotes = quotes
        self.quoteUrl = quoteUrl
        self.fiatTransactions = fiatTransactions
    }

    public func getQuotes(walletId _: String, type _: FiatQuoteType, assetId _: AssetId, request _: FiatQuoteRequest) async throws -> [FiatQuote] {
        quotes
    }

    public func getQuoteUrl(walletId _: String, quoteId _: String) async throws -> FiatQuoteUrl {
        quoteUrl
    }

    public func getFiatTransactions(walletId _: String) async throws -> [FiatTransactionInfo] {
        fiatTransactions
    }
}
