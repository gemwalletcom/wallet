// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import protocol Gemstone.GemSwapServiceProtocol
import struct Gemstone.SwapperQuote
import Primitives

public extension GemSwapServiceProtocol {
    func supportedAssets(for assetId: Primitives.AssetId) -> ([Primitives.Chain], [Primitives.AssetId]) {
        let assetList = supportedAssets(assetId: assetId.identifier)
        return (
            assetList.chains.compactMap { try? $0.map() },
            assetList.assetIds.compactMap { try? Primitives.AssetId(id: $0) },
        )
    }

    func getQuotes(
        wallet: Primitives.Wallet,
        fromAsset: Asset,
        toAsset: Asset,
        amount: BigInt,
        useMaxAmount: Bool,
        slippage: SwapSlippage,
    ) async throws -> [SwapperQuote] {
        let quotes = try await getQuotes(
            wallet: wallet.json(),
            fromAsset: fromAsset.json(),
            toAsset: toAsset.json(),
            value: amount.description,
            useMaxAmount: useMaxAmount,
            slippageBps: slippage.exactBps,
        )
        try Task.checkCancellation()
        return quotes
    }

    func getQuoteData(wallet: Primitives.Wallet, quote: SwapperQuote) async throws -> Primitives.SwapQuoteData {
        try await Primitives.SwapQuoteData(getQuoteData(wallet: wallet.json(), quote: quote))
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
