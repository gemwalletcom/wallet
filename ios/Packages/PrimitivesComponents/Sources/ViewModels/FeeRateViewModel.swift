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
    public let priority: FeePriority
    public let unitValue: BigInt
    public let fee: BigInt?
    public let unitType: FeeUnitType
    public let decimals: Int
    public let symbol: String

    public init(
        priority: FeePriority,
        unitValue: BigInt,
        fee: BigInt?,
        unitType: FeeUnitType,
        decimals: Int,
        symbol: String,
    ) {
        self.priority = priority
        self.unitValue = unitValue
        self.fee = fee
        self.unitType = unitType
        self.decimals = decimals
        self.symbol = symbol
    }

    public var id: String {
        priority.rawValue
    }

    public var emoji: String {
        switch priority {
        case .fast: Emoji.FeeRate.fast.rawValue
        case .normal: Emoji.FeeRate.normal.rawValue
        }
    }

    public var title: String {
        switch priority {
        case .normal: Localized.FeeRates.normal
        case .fast: Localized.FeeRates.fast
        }
    }

    public var feeUnitModel: FeeUnitViewModel {
        FeeUnitViewModel(
            unit: FeeUnit(type: unitType, value: unitValue),
            decimals: decimals,
            symbol: symbol,
        )
    }

    public var valueText: String {
        feeUnitModel.value
    }
}
