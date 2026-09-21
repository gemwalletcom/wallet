// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import struct Gemstone.GemSwapPairFailure
import struct Gemstone.GemSwapQuoteInput
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.GemTransferData
import struct Gemstone.SwapperQuote
import Primitives

public extension GemSwapQuoteServiceProtocol {
    var currency: Primitives.Currency {
        getCurrency().toPrimitives()
    }

    var slippage: SwapSlippage {
        switch slippageBps() {
        case let .some(bps): .manual(bps: bps)
        case .none: .auto
        }
    }

    func setSlippage(_ slippage: SwapSlippage) throws {
        try setSlippageBps(bps: slippage.exactBps)
    }

    func getQuotes(fromAsset: Asset, toAsset: Asset, input: GemSwapQuoteInput) async throws -> [SwapperQuote] {
        let quotes = try await getQuotes(
            fromAsset: fromAsset.toGem(),
            toAsset: toAsset.toGem(),
            value: input.request.value,
            useMaxAmount: input.useMaxAmount,
            slippageBps: input.request.slippageBps,
        )
        try Task.checkCancellation()
        return quotes
    }

    func getTransferData(fromAsset: Asset, toAsset: Asset, quote: SwapperQuote) async throws -> GemTransferData {
        try await getTransfer(quote: quote).transferData(fromAsset: fromAsset.toGem(), toAsset: toAsset.toGem())
    }

    func refreshPair(assetIds: [Primitives.AssetId]) async -> [GemSwapPairFailure] {
        await refreshPair(assetIds: assetIds.ids)
    }
}

private extension SwapSlippage {
    var exactBps: UInt32? {
        switch self {
        case .auto: nil
        case let .manual(bps): bps
        }
    }
}
