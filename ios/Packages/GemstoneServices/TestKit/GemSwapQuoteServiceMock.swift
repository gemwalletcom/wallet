// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import typealias Gemstone.Currency
import enum Gemstone.GemSlippageCheck
import class Gemstone.GemSwapQuoteSummary
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.GemSwapTransfer
import struct Gemstone.SwapperAssetList
import enum Gemstone.SwapProvider
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapperSlippage
import GemstonePrimitives
import Primitives
import PrimitivesTestKit

public final class GemSwapQuoteServiceMock: GemSwapQuoteServiceProtocol, @unchecked Sendable {
    private let quotes: @Sendable (BigInt) -> [SwapperQuote]
    private let quoteData: Primitives.SwapQuoteData
    private let assetList: SwapperAssetList
    private let quotesDelay: Duration?
    private let quotesError: Error?
    private let pairSuggestion: GemSwapPairSuggestion?
    private let slippageCheckResult: GemSlippageCheck
    public private(set) var storedSlippageBps: UInt32?

    public init(
        quotes: @escaping @Sendable (BigInt) -> [SwapperQuote],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
        slippageBps: UInt32? = nil,
        slippageCheck: GemSlippageCheck = .valid,
    ) {
        self.quotes = quotes
        self.quoteData = quoteData
        self.assetList = assetList
        self.quotesDelay = quotesDelay
        self.quotesError = quotesError
        self.pairSuggestion = pairSuggestion
        storedSlippageBps = slippageBps
        slippageCheckResult = slippageCheck
    }

    public convenience init(
        quotes: [SwapperQuote] = [.mock()],
        quoteData: Primitives.SwapQuoteData = .mock(),
        assetList: SwapperAssetList = .mock(),
        quotesDelay: Duration? = nil,
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
        slippageBps: UInt32? = nil,
        slippageCheck: GemSlippageCheck = .valid,
    ) {
        self.init(
            quotes: { _ in quotes },
            quoteData: quoteData,
            assetList: assetList,
            quotesDelay: quotesDelay,
            quotesError: quotesError,
            pairSuggestion: pairSuggestion,
            slippageBps: slippageBps,
            slippageCheck: slippageCheck,
        )
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.rawValue
    }

    public func slippageBps() -> UInt32? {
        storedSlippageBps
    }

    public func setSlippageBps(bps: UInt32?) throws {
        storedSlippageBps = bps
    }

    public func slippageCheck(bps _: UInt32) -> GemSlippageCheck {
        slippageCheckResult
    }

    public func defaultSlippage(chain _: Chain) -> SwapperSlippage {
        SwapperSlippage(bps: 100, mode: .auto)
    }

    public func refreshIntervalMilliseconds() -> UInt64 {
        30_000
    }

    public func quoteDebounceMilliseconds() -> UInt64 {
        250
    }

    public func updateBalances(assetIds _: [AssetId]) async throws {}

    public func addPrices(assetIds _: [AssetId]) async throws {}

    public func selectedQuote(quotes: [SwapperQuote], preferred: SwapProvider?) -> SwapperQuote? {
        quotes.first(where: { $0.data.provider.id == preferred }) ?? quotes.first
    }

    public func getQuotes(fromAsset _: Asset, toAsset _: Asset, value: BigUInt, useMaxAmount _: Bool, slippageBps _: UInt32?) async throws -> [SwapperQuote] {
        if let quotesDelay {
            try await Task.sleep(for: quotesDelay)
        }
        if let quotesError {
            throw quotesError
        }
        return quotes(BigInt(value))
    }

    public func getTransfer(quote: SwapperQuote) async throws -> GemSwapTransfer {
        try GemSwapTransfer(
            quote: GemSwapQuoteSummary.fromQuote(quote: quote).quote(),
            data: quoteData.json(),
            recipient: quote.request.destinationAddress,
            value: quote.request.value,
            useMaxAmount: quote.request.options.useMaxAmount,
        )
    }

    public func supportedAssets(assetId _: AssetId) -> SwapperAssetList {
        assetList
    }

    public func suggestPair(payAssetId _: AssetId?) async throws -> GemSwapPairSuggestion? {
        pairSuggestion
    }
}
