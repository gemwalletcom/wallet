// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct FeeRateViewModel: Identifiable {
    public let feeRate: FeeRate
    public let unitType: FeeUnitType
    public let decimals: Int
    public let symbol: String
    public let totalFee: BigInt

    public init(
        feeRate: FeeRate,
        unitType: FeeUnitType,
        decimals: Int,
        symbol: String,
        totalFee: BigInt,
    ) {
        self.feeRate = feeRate
        self.unitType = unitType
        self.decimals = decimals
        self.symbol = symbol
        self.totalFee = totalFee
    }

    public var id: String {
        feeRate.priority.rawValue
    }

    public var emoji: String {
        switch feeRate.priority {
        case .fast: Emoji.FeeRate.fast.rawValue
        case .normal: Emoji.FeeRate.normal.rawValue
        }
    }

    public var title: String {
        switch feeRate.priority {
        case .normal: Localized.FeeRates.normal
        case .fast: Localized.FeeRates.fast
        }
    }

    public var feeUnitModel: FeeUnitViewModel {
        let unit = FeeUnit(type: unitType, value: totalFee)
        return FeeUnitViewModel(
            unit: unit,
            decimals: decimals,
            symbol: symbol,
        )
    }

    public var valueText: String {
        feeUnitModel.value
    }
}

