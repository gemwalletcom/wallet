// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SwapperQuote
import Primitives
import PrimitivesTestKit
import GemstoneServices

public extension SwapQuoteDataProvider {
    static func mock() -> SwapQuoteDataProviderMock {
        SwapQuoteDataProviderMock()
    }
}

public extension SwapQuoteDataProvidable where Self == SwapQuoteDataProviderMock {
    static func mock() -> SwapQuoteDataProviderMock {
        SwapQuoteDataProviderMock()
    }
}

public struct SwapQuoteDataProviderMock: SwapQuoteDataProvidable {
    public let quoteData: Primitives.SwapQuoteData

    public init(quoteData: Primitives.SwapQuoteData = .mock()) {
        self.quoteData = quoteData
    }

    public func fetchQuoteData(wallet _: Wallet, quote _: SwapperQuote) async throws -> Primitives.SwapQuoteData {
        quoteData
    }
}
