// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import typealias Gemstone.Asset
import typealias Gemstone.AssetId
import typealias Gemstone.Chain
import typealias Gemstone.Currency
import struct Gemstone.GemNumberFormat
import enum Gemstone.GemSlippageSelection
import struct Gemstone.GemSlippageSession
import struct Gemstone.GemSwapPairSelection
import struct Gemstone.GemSwapPairSuggestion
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.GemSwapQuoteSummary
import struct Gemstone.GemSwapSession
import enum Gemstone.GemSwapSide
import struct Gemstone.GemSwapTransfer
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapperSlippage
import func Gemstone.swapQuote
import struct Gemstone.SwapQuoteData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public final class GemSwapQuoteServiceMock: GemSwapQuoteServiceProtocol, @unchecked Sendable {
    private let quotes: @Sendable (BigInt) -> [SwapperQuote]
    private let quoteData: Gemstone.SwapQuoteData
    private let quotesError: Error?
    private let pairSuggestion: GemSwapPairSuggestion?
    public private(set) var storedSlippageBps: UInt32?
    public private(set) var priceSubscriptions: [[AssetId]] = []

    public init(
        quotes: @escaping @Sendable (BigInt) -> [SwapperQuote],
        quoteData: Gemstone.SwapQuoteData = .mock(),
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
        slippageBps: UInt32? = nil,
    ) {
        self.quotes = quotes
        self.quoteData = quoteData
        self.quotesError = quotesError
        self.pairSuggestion = pairSuggestion
        storedSlippageBps = slippageBps
    }

    public convenience init(
        quotes: [SwapperQuote] = [.mock()],
        quoteData: Gemstone.SwapQuoteData = .mock(),
        quotesError: Error? = nil,
        pairSuggestion: GemSwapPairSuggestion? = nil,
        slippageBps: UInt32? = nil,
    ) {
        self.init(
            quotes: { _ in quotes },
            quoteData: quoteData,
            quotesError: quotesError,
            pairSuggestion: pairSuggestion,
            slippageBps: slippageBps,
        )
    }

    public func getCurrency() -> Currency {
        Primitives.Currency.usd.toGem()
    }

    public func newSession() -> GemSwapSession {
        GemSwapSession(quotePhase: .noInput, transferPhase: .idle)
    }

    public func slippageBps() -> UInt32? {
        storedSlippageBps
    }

    public func setSlippageBps(bps: UInt32?) throws {
        storedSlippageBps = bps
    }

    public func newSlippageSession(selection: GemSlippageSelection) -> GemSlippageSession {
        switch selection {
        case .auto: GemSlippageSession(isAuto: true, bps: 0)
        case let .manual(bps): GemSlippageSession(isAuto: false, bps: bps)
        }
    }

    public func amountForPercent(available: BigInt, percent: UInt32) -> BigInt {
        available * BigInt(percent) / BigInt(100)
    }

    public func slippageBpsFromPercent(percent: Double) -> UInt32? {
        percent > 0 ? UInt32((percent * 100).rounded()) : .none
    }

    public func slippagePercent(bps: UInt32) -> Double {
        Double(bps) / 100
    }

    public func slippagePercentText(bps: UInt32, format: GemNumberFormat) -> String {
        (Decimal(bps) / 100).description.replacingOccurrences(of: ".", with: format.decimalSeparator)
    }

    public func selectPairAsset(selection: GemSwapPairSelection, side: GemSwapSide, assetId: String) -> GemSwapPairSelection {
        switch side {
        case .pay: GemSwapPairSelection(payAssetId: assetId, receiveAssetId: selection.receiveAssetId)
        case .receive: GemSwapPairSelection(payAssetId: selection.payAssetId, receiveAssetId: assetId)
        }
    }

    public func defaultSlippage(chain _: Chain) -> SwapperSlippage {
        SwapperSlippage(bps: 100, mode: .auto)
    }

    public func refreshIntervalMilliseconds() -> UInt64 {
        30000
    }

    public func quoteDebounceMilliseconds() -> UInt64 {
        250
    }

    public func updateBalances(assetIds _: [AssetId]) async throws {}

    public func addPrices(assetIds: [AssetId]) async throws {
        priceSubscriptions.append(assetIds)
    }

    public func getQuotes(fromAsset _: Asset, toAsset _: Asset, value: BigUInt, useMaxAmount _: Bool, slippageBps _: UInt32?) async throws -> [SwapperQuote] {
        if let quotesError {
            throw quotesError
        }
        return quotes(BigInt(value))
    }

    public func getTransfer(quote: SwapperQuote) async throws -> GemSwapTransfer {
        GemSwapTransfer(
            quote: Gemstone.swapQuote(quote: quote),
            data: quoteData,
            recipient: quote.request.destinationAddress,
            value: quote.request.value,
            useMaxAmount: quote.request.options.useMaxAmount,
        )
    }

    public func suggestPair(payAssetId _: AssetId?) async -> GemSwapPairSuggestion? {
        pairSuggestion
    }
}
