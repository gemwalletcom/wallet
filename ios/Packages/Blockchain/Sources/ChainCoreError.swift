// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public enum ChainCoreError: String, Error, Equatable {
    case feeRateMissed
    case cantEstimateFee
    case incorrectAmount
    case dustThreshold
    case insufficientBalance

    public static func fromError(_ error: Error) -> ChainCoreError? {
        let description = error.localizedDescription
        if description.contains("dust threshold") {
            return .dustThreshold
        }
        if description.contains("insufficient balance") {
            return .insufficientBalance
        }
        for errorCase in [ChainCoreError.feeRateMissed, .cantEstimateFee, .incorrectAmount] {
            if description.contains(errorCase.rawValue) {
                return errorCase
            }
        }

        return nil
    }
}
