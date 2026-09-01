// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import protocol Gemstone.GemSwapServiceProtocol
import struct Gemstone.SwapperQuote
import GemstonePrimitives
import Primitives
import struct Gemstone.GemTransferData

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
            fromAsset: fromAsset.map(),
            toAsset: toAsset.map(),
            value: amount.description,
            useMaxAmount: useMaxAmount,
            slippageBps: slippage.exactBps,
        )
        try Task.checkCancellation()
        return quotes
    }

    func getTransferData(wallet: Primitives.Wallet, fromAsset: Asset, toAsset: Asset, quote: SwapperQuote) async throws -> GemTransferData {
        try GemTransferData(swap: await getTransfer(wallet: wallet.json(), quote: quote), fromAsset: fromAsset, toAsset: toAsset)
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
