// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import Foundation
import Primitives

public enum ChainCoreError: String, Error, Equatable {
    case feeRateMissed
    case cantEstimateFee
    case incorrectAmount
    case dustThreshold
    case insufficientBalance

    public static func fromError(_ error: Error) -> ChainCoreError? {
        if let signerError = Self.signerError(error) {
            return switch signerError {
            case .dustThreshold: .dustThreshold
            case .insufficientFunds: .insufficientBalance
            case .invalidInput,
                 .signingError,
                 .swapValueBelowMinimum: nil
            }
        }

        let description = error.localizedDescription
        for errorCase in [ChainCoreError.feeRateMissed, .cantEstimateFee, .incorrectAmount] {
            if description.contains(errorCase.rawValue) {
                return errorCase
            }
        }

        return nil
    }

    private static func signerError(_ error: Error) -> GemSignerError? {
        if case let GemstoneError.SignerError(signerError, _) = error {
            return signerError
        }
        if case let GemConfirmError.Sign(signerError, _) = error {
            return signerError
        }
        return nil
    }
}
