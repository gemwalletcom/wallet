// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Localization
import SwiftUI

extension ChainCoreError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .cantEstimateFee, .feeRateMissed: Localized.Errors.unableEstimateNetworkFee
        case .incorrectAmount: Localized.Errors.invalidAmount
        case .dustThreshold: Localized.Errors.dustThresholdShort
        }
    }
}
