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
        var gemOptions: [Gemstone.FeeOption: String] = [:]
        for (option, value) in feeOptionMap {
            switch option {
            case .tokenAccountCreation:
                gemOptions[.tokenAccountCreation] = value.description
            }
        }
        return GemFeeOptions(options: gemOptions)
    }

    func map() throws -> FeeOptionMap {
        var feeOptions: FeeOptionMap = [:]
        for (option, value) in options {
            let feeOption: Primitives.FeeOption = switch option {
            case .tokenAccountCreation:
                .tokenAccountCreation
            }
            feeOptions[feeOption] = try BigInt.from(string: value)
        }
        return feeOptions
    }
}

public extension FeeOptionMap {
    func map() -> GemFeeOptions {
        GemFeeOptions.from(self)
    }
}
