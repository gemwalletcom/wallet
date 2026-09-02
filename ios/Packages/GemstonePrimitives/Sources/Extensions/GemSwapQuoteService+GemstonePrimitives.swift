// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.SwapperQuote
import Primitives
import struct Gemstone.GemTransferData

public extension GemSwapQuoteServiceProtocol {
    var currencyCode: String {
        currency()
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

    func supportedAssets(for assetId: Primitives.AssetId) -> ([Primitives.Chain], [Primitives.AssetId]) {
        let assetList = supportedAssets(assetId: assetId.identifier)
        return (
            assetList.chains.compactMap { try? $0.map() },
            assetList.assetIds.compactMap { try? Primitives.AssetId(id: $0) },
        )
    }

    func getQuotes(
        fromAsset: Asset,
        toAsset: Asset,
        amount: BigInt,
        useMaxAmount: Bool,
        slippage: SwapSlippage,
    ) async throws -> [SwapperQuote] {
        let quotes = try await getQuotes(
            fromAsset: fromAsset.map(),
            toAsset: toAsset.map(),
            value: amount.description,
            useMaxAmount: useMaxAmount,
            slippageBps: slippage.exactBps,
        )
        try Task.checkCancellation()
        return quotes
    }

    func getTransferData(fromAsset: Asset, toAsset: Asset, quote: SwapperQuote) async throws -> GemTransferData {
        try await getTransfer(quote: quote).transferData(fromAsset: fromAsset.map(), toAsset: toAsset.map())
    }

    func updateBalances(assetIds: [Primitives.AssetId]) async throws {
        try await updateBalances(assetIds: assetIds.ids)
    }

    func addPrices(assetIds: [Primitives.AssetId]) async throws {
        try await addPrices(assetIds: assetIds.ids)
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
