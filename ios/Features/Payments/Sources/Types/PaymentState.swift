// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives

public struct PaymentState: Sendable {
    public var quotes: PaymentQuotes
    public var refresh: StateViewType<PaymentQuotes> = .noData
    public var transferData: StateViewType<TransferData> = .noData
    public var selectedQuoteId: String?
    public var collectedQuoteIds: Set<String> = []

    public init(quotes: PaymentQuotes) {
        self.quotes = quotes
        selectedQuoteId = quotes.quotes.first?.id
    }
}

public extension PaymentState {
    internal mutating func replace(with quotes: PaymentQuotes) {
        self = PaymentState(quotes: quotes)
    }

    internal mutating func select(quoteId: String?) {
        selectedQuoteId = quoteId
    }

    internal mutating func completeDataCollection() {
        guard let selectedQuote else { return }
        collectedQuoteIds.insert(selectedQuote.id)
    }

    var selectedQuote: PaymentQuote? {
        quotes.quotes.first { $0.id == selectedQuoteId }
    }

    internal var needsDataCollection: Bool {
        guard let selectedQuote, selectedQuote.collectDataUrl != nil else { return false }
        return !collectedQuoteIds.contains(selectedQuote.id)
    }

    var isLoading: Bool {
        refresh.isLoading || transferData.isLoading
    }

    var error: (any Error)? {
        if case let .error(error) = transferData {
            return error
        }
        if case let .error(error) = refresh {
            return error
        }
        return nil
    }
}
