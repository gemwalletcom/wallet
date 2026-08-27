// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import protocol Gemstone.GemSwapServiceProtocol
import typealias Gemstone.SwapQuoteData
import struct Gemstone.SwapperAssetList
import struct Gemstone.SwapperQuote
import typealias Gemstone.Wallet
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemSwapServiceMock: GemSwapServiceProtocol, @unchecked Sendable {
    private let quotes: @Sendable (BigInt) -> [SwapperQuote]
    private let quoteData: Primitives.SwapQuoteData
    private let assetList: SwapperAssetList
    private let quotesDelay: Duration?
    private let quotesError: Error?

    public init(
        quotes: @escaping @Sendable (BigInt) -> [SwapperQuote],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
    ) {
        self.quotes = quotes
        self.quoteData = quoteData
        self.assetList = assetList
        self.quotesDelay = quotesDelay
        self.quotesError = quotesError
    }

    public convenience init(
        quotes: [SwapperQuote] = [.mock()],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
    ) {
        self.init(
            quotes: { _ in quotes },
            quoteData: quoteData,
            assetList: assetList,
            quotesDelay: quotesDelay,
            quotesError: quotesError,
        )
    }

    public func getQuotes(wallet _: Wallet, fromAsset _: Asset, toAsset _: Asset, value: String, useMaxAmount _: Bool, slippageBps _: UInt32?) async throws -> [SwapperQuote] {
        if let quotesDelay {
            try await Task.sleep(for: quotesDelay)
        }
        if let quotesError {
            throw quotesError
        }
        return try quotes(BigInt.from(string: value))
    }

    public func getQuoteData(wallet _: Wallet, quote _: SwapperQuote) async throws -> SwapQuoteData {
        try quoteData.json()
    }

    public func supportedAssets(assetId _: AssetId) -> SwapperAssetList {
        assetList
    }
}
