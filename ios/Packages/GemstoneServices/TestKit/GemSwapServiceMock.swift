// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import protocol Gemstone.GemSwapServiceProtocol
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.GemSwapTransfer
import struct Gemstone.SwapperAssetList
import struct Gemstone.SwapperQuote
import func Gemstone.swapQuote
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
    private let pairSuggestion: GemSwapPairSuggestion?

    public init(
        quotes: @escaping @Sendable (BigInt) -> [SwapperQuote],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
    ) {
        self.quotes = quotes
        self.quoteData = quoteData
        self.assetList = assetList
        self.quotesDelay = quotesDelay
        self.quotesError = quotesError
        self.pairSuggestion = pairSuggestion
    }

    public convenience init(
        quotes: [SwapperQuote] = [.mock()],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
    ) {
        self.init(
            quotes: { _ in quotes },
            quoteData: quoteData,
            assetList: assetList,
            quotesDelay: quotesDelay,
            quotesError: quotesError,
            pairSuggestion: pairSuggestion,
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

    public func getTransfer(wallet _: Wallet, quote: SwapperQuote) async throws -> GemSwapTransfer {
        try GemSwapTransfer(
            quote: swapQuote(quote: quote),
            data: quoteData.json(),
            recipient: quote.request.destinationAddress,
            value: quote.request.value,
            useMaxAmount: quote.request.options.useMaxAmount,
        )
    }

    public func supportedAssets(assetId _: AssetId) -> SwapperAssetList {
        assetList
    }

    public func suggestPair(walletId _: String, payAssetId _: AssetId?) async throws -> GemSwapPairSuggestion? {
        pairSuggestion
    }
}
