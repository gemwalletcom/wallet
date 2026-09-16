// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapQuotesResult
import struct Gemstone.GemSwapRequest
import struct Gemstone.GemSwapSession
import enum Gemstone.SwapperError
import struct Gemstone.SwapperQuote
import Primitives
import PrimitivesTestKit

public extension GemSwapRequest {
    static let mock = GemSwapRequest(
        payAssetId: AssetId.mockEthereum().identifier,
        receiveAssetId: AssetId.mockEthereumUSDT().identifier,
        value: 1_000_000_000_000_000_000,
        slippageBps: nil,
    )
}

public extension GemSwapSession {
    static func mock() -> GemSwapSession {
        GemSwapSession(quotePhase: .noInput, transferPhase: .idle)
    }

    static func mockLoading() -> GemSwapSession {
        mock().onRequestChanged(request: .mock)
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
