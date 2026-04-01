// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Localization
import Primitives

extension Gemstone.GatewayError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .NetworkError(string): string
        case let .PlatformError(string): string
        }
    }
}

extension Gemstone.GemstoneError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .AnyError(string): string
        }
    }
}

extension Gemstone.SwapperError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .NotSupportedChain: Localized.Errors.Swap.notSupportedChain
        case .NotSupportedAsset: Localized.Errors.Swap.notSupportedAsset
        case .NoQuoteAvailable: Localized.Errors.Swap.noQuoteAvailable
        case .NoAvailableProvider: Localized.Errors.Swap.notSupportedPair
        case .InputAmountError: Localized.Errors.Swap.amountTooSmall
        case let .ComputeQuoteError(error),
             let .TransactionError(error): error
        case .InvalidRoute: "Invalid Route"
        }
    }
}

extension Gemstone.AlienError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .RequestError(msg: msg): msg
        case let .ResponseError(msg: msg): msg
        case let .Http(status, _): "Response Status: \(status)"
        }
    }
}
