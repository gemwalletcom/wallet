// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Localization
import Primitives

extension Gemstone.GatewayError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .Offline: Localized.Errors.networkOffline
        case let .NetworkError(string): string
        case let .PlatformError(string): string
        }
    }
}

extension Gemstone.GemstoneError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .AnyError(string): string
        case let .SignerError(_, msg): msg
        }
    }
}

extension Gemstone.GemPaymentError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .NoPaymentOptions: Localized.Errors.notSupported
        case let .InvalidRequest(reason), let .Network(reason): reason
        }
    }
}

extension Gemstone.GemWalletConnectError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .UnsupportedChains: Localized.Errors.Connections.unsupportedChain
        case .InvalidOrigin: Localized.Errors.Connections.maliciousOrigin
        case .UnsupportedWallets: Localized.Errors.Connections.noSupportedWallets
        case let .Service(msg): msg
        }
    }
}

extension Gemstone.SwapperError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .NotSupportedChain, .NotSupportedAsset:
            Localized.Errors.Swap.notSupportedAsset
        case .NoQuoteAvailable, .NoAvailableProvider, .InvalidRoute,
             .ComputeQuoteError, .TransactionError:
            Localized.Errors.Swap.noQuoteAvailable
        case .InputAmountError: Localized.Errors.Swap.amountTooSmall
        }
    }
}

extension Gemstone.AlienError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case let .RequestError(msg: msg): msg
        case let .ResponseError(msg: msg): msg
        case let .Http(status, _): "Response Status: \(status)"
        case .Offline: Localized.Errors.networkOffline
        }
    }
}
