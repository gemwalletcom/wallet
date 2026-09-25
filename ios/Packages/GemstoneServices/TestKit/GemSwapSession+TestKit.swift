// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapQuoteInput
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import Primitives
import PrimitivesTestKit

public extension GemSwapRequest {
    static let mock = GemSwapRequest(
        payAssetId: AssetId.mock(chain: .ethereum).identifier,
        receiveAssetId: AssetId.mock(chain: .ethereum, tokenId: "0xdAC17F958D2ee523a2206206994597C13D831ec7").identifier,
        value: 1_000_000_000_000_000_000,
        slippageBps: nil,
    )
}

public extension GemSwapSession {
    static func mock() -> GemSwapSession {
        GemSwapSession(quotePhase: .noInput, transferPhase: .idle)
    }

    static func mockLoading() -> GemSwapSession {
        GemSwapSession(quotePhase: .loading(request: .mock), transferPhase: .idle, input: GemSwapQuoteInput(request: .mock, useMaxAmount: false))
    }

    static func mockReady(quotes: [SwapperQuote] = [.mock()]) -> GemSwapSession {
        mockLoading().onQuoteResults(results: GemSwapQuotesResult(request: .mock, quotes: quotes, error: nil))
    }

    static func mockFailed(_ error: SwapperError) -> GemSwapSession {
        mockLoading().onQuoteResults(results: GemSwapQuotesResult(request: .mock, quotes: [], error: error))
    }

    func failedTransfer(_ error: SwapperError) -> GemSwapSession {
        let started = startTransfer()!
        return started.onTransferFailed(transfer: started.transferPhase, error: error)
    }
}
