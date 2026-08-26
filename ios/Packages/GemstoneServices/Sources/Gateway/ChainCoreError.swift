// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

import GemstonePrimitives

public enum ChainCoreError: String, Error, Equatable {
    case feeRateMissed
    case cantEstimateFee
    case incorrectAmount
    case dustThreshold
    case insufficientBalance

    public static func fromError(_ error: Error) -> ChainCoreError? {
        if case let GemstoneError.SignerError(error: signerError, msg: _) = error {
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
}
