// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AlienError
import func Gemstone.alienErrorText
import enum Gemstone.GatewayError
import enum Gemstone.GemAddNodeError
import enum Gemstone.GemPaymentError
import enum Gemstone.GemServiceError
import enum Gemstone.GemstoneError
import enum Gemstone.GemWalletConnectError
import func Gemstone.paymentErrorText
import func Gemstone.swapErrorDisplay
import enum Gemstone.SwapperError
import Localization
import Primitives

extension KeystoreError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .missingPassword, .missingLockKey: Localized.Errors.keystoreAccess
        }
    }
}

extension Gemstone.GemServiceError: @retroactive LocalizedError {
    public var errorDescription: String? {
        text().text
    }
}

extension Gemstone.GatewayError: @retroactive LocalizedError {
    public var errorDescription: String? {
        text().text
    }
}

extension Gemstone.GemAddNodeError: @retroactive LocalizedError {
    public var errorDescription: String? {
        text().text
    }
}

extension Gemstone.GemstoneError: @retroactive LocalizedError {
    public var errorDescription: String? {
        text().text
    }
}

extension Gemstone.GemPaymentError: @retroactive LocalizedError {
    public var errorDescription: String? {
        paymentErrorText(error: self).text
    }
}

extension Gemstone.GemWalletConnectError: @retroactive LocalizedError {
    public var errorDescription: String? {
        text().text
    }
}

extension Gemstone.SwapperError: @retroactive LocalizedError {
    public var errorDescription: String? {
        swapErrorDisplay(error: self, payAsset: nil).errorDescription
    }
}

extension Gemstone.AlienError: @retroactive LocalizedError {
    public var errorDescription: String? {
        alienErrorText(error: self).text
    }
}
