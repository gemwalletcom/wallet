// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension NumberFormatStyleConfiguration.Precision {
    static let twoPlaces = fractionLength(2)
    static let upToTwoPlaces = fractionLength(0 ... 2)
    static let upToFourPlaces = fractionLength(0 ... 4)
    static let fourSignificant = significantDigits(1 ... 4)
    static let full = fractionLength(0 ... 32)

    static func adaptive(for magnitude: Double) -> Self {
        switch magnitude {
        case dustThreshold ..< smallValueThreshold: .fourSignificant
        default: .twoPlaces
        }
    }

    private static let smallValueThreshold: Double = 0.99
    private static let dustThreshold: Double = 1e-10
}
