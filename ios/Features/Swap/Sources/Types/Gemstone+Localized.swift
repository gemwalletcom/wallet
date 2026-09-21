// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemSlippageCheck
import enum Gemstone.GemSwapButtonAction
import enum Gemstone.GemSwapDetailRow
import enum Gemstone.GemSwapErrorDisplay
import GemstonePrimitives
import Localization
import Primitives

extension GemSwapButtonAction {
    func title(symbol: String) -> String {
        switch self {
        case .retryQuote, .retryTransfer: Localized.Common.tryAgain
        case .insufficientBalance: Localized.Transfer.insufficientBalance(symbol)
        case .useMinimumAmount: Localized.Swap.useMinimumAmount
        case .swap: Localized.Wallet.swap
        }
    }
}

extension GemSwapDetailRow {
    var title: String {
        switch self {
        case .provider: Localized.Common.provider
        case .rate: Localized.Buy.rate
        case .estimatedTime: Localized.Swap.EstimatedTime.title
        case .priceImpact: Localized.Swap.priceImpact
        case .minimumReceive: Localized.Swap.minReceive
        case .slippage: Localized.Swap.slippage
        }
    }
}

extension GemSwapErrorDisplay: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .notSupportedAsset: Localized.Errors.Swap.notSupportedAsset
        case .noQuote: Localized.Errors.Swap.noQuoteAvailable
        case .offline: Localized.Errors.networkOffline
        case let .minimumAmount(asset, minAmount):
            Localized.Errors.Swap.minimumAmount(
                ValueFormatter(style: .auto).string(minAmount, asset: asset.toPrimitives()).boldMarkdown(),
            )
        case .amountTooSmall: Localized.Errors.Swap.amountTooSmall
        }
    }
}

extension GemSlippageCheck {
    func errorText(minimum: GemFormattedNumber, maximum: GemFormattedNumber) -> String? {
        switch self {
        case .belowMinimum: Localized.Common.minimumValue(minimum.text())
        case .aboveMaximum: Localized.Common.maximumValue(maximum.text())
        case .valid, .high: nil
        }
    }
}
