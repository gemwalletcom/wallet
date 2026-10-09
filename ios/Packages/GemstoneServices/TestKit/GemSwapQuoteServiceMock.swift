// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import typealias Gemstone.AssetId
import typealias Gemstone.Currency
import struct Gemstone.GemSwapPairFailure
import struct Gemstone.GemSwapPairSelection
import struct Gemstone.GemSwapPairSuggestion
import struct Gemstone.GemSwapQuoteInput
import protocol Gemstone.GemSwapQuoteServiceProtocol
import struct Gemstone.GemSwapQuoteSummary
import struct Gemstone.GemSwapSession
import enum Gemstone.GemSwapSide
import struct Gemstone.GemTransferData
import struct Gemstone.SwapData
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapProviderData
import struct Gemstone.SwapQuote
import struct Gemstone.SwapQuoteData
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit

public final class GemSwapQuoteServiceMock: GemSwapQuoteServiceProtocol, @unchecked Sendable {
    public var isAvailableValue = true

    public func isAvailable() -> Bool {
        isAvailableValue
    }

    private let quotes: @Sendable (BigInt) async throws -> [SwapperQuote]
    private let quoteData: Gemstone.SwapQuoteData
    private let quotesError: Error?
    public var transferError: Error?
    private let pairSuggestion: GemSwapPairSuggestion?
    public private(set) var storedSlippageBps: UInt32?
    public private(set) var priceSubscriptions: [[AssetId]] = []
    public private(set) var balanceUpdates: [[AssetId]] = []

    public init(
        quotes: @escaping @Sendable (BigInt) async throws -> [SwapperQuote],
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

    public func amountForPercent(available: BigInt, percent: UInt32) -> BigInt {
        available * BigInt(percent) / BigInt(100)
    }

    public func selectPairAsset(selection: GemSwapPairSelection, side: GemSwapSide, assetId: AssetId) -> GemSwapPairSelection {
        switch side {
        case .pay: GemSwapPairSelection(payAssetId: assetId, receiveAssetId: selection.receiveAssetId)
        case .receive: GemSwapPairSelection(payAssetId: selection.payAssetId, receiveAssetId: assetId)
        }
    }

    public func refreshPair(assetIds: [AssetId]) async -> [GemSwapPairFailure] {
        priceSubscriptions.append(assetIds)
        balanceUpdates.append(assetIds)
        return []
    }

    public func getQuotes(input: GemSwapQuoteInput) async throws -> [SwapperQuote] {
        if let quotesError {
            throw quotesError
        }
        return try await quotes(BigInt(input.request.value))
    }

    public func transferData(quote: SwapperQuote) async throws -> GemTransferData {
        if let transferError {
            throw transferError
        }
        return GemTransferData.mock(
            inputType: .swap(fromAsset: Asset.mock().toGem(), toAsset: Asset.mock().toGem(), swapData: SwapData(quote: quote.swapQuote, data: quoteData)),
            recipient: .mock(address: quote.request.destinationAddress),
            value: BigInt(quote.request.value),
            useMaxAmount: quote.request.options.useMaxAmount,
        )
    }

    public func suggestPair(payAssetId _: AssetId?) async -> GemSwapPairSuggestion? {
        pairSuggestion
    }
}

private extension SwapperQuote {
    var swapQuote: SwapQuote {
        SwapQuote(
            fromAddress: request.walletAddress,
            fromValue: fromValue,
            minFromValue: minFromValue,
            toAddress: request.destinationAddress,
            toValue: toValue,
            providerData: SwapProviderData(provider: data.provider.id, name: data.provider.name, protocolName: data.provider.protocol),
            slippageBps: data.slippageBps,
            slippageMode: request.options.slippage.mode,
            etaInSeconds: etaInSeconds,
            useMaxAmount: request.options.useMaxAmount,
        )
    }
}
