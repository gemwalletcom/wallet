// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Gemstone
import Primitives

public extension GemFeeOptions {
    static func empty() -> GemFeeOptions {
        GemFeeOptions(options: [:])
    }

    static func from(_ feeOptionMap: FeeOptionMap) -> GemFeeOptions {
        var gemOptions: [Gemstone.FeeOption: BigInt] = [:]
        for (option, value) in feeOptionMap {
            switch option {
            case .tokenAccountCreation:
                gemOptions[.tokenAccountCreation] = value
            }
        }
        return GemFeeOptions(options: gemOptions)
    }

    func map() -> FeeOptionMap {
        var feeOptions: FeeOptionMap = [:]
        for (option, value) in options {
            let feeOption: Primitives.FeeOption = switch option {
            case .tokenAccountCreation:
                .tokenAccountCreation
            }
            feeOptions[feeOption] = value
        }
        return feeOptions
    }
}

public extension FeeOptionMap {
    func map() -> GemFeeOptions {
        GemFeeOptions.from(self)
    }
}
