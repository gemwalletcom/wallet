// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import Localization
import Primitives

public struct FeeUnitViewModel {
    private let unit: FeeUnit
    private let decimals: Int
    private let symbol: String
    private let numericFormatter: NumericFormatter
    private let valueFormatter = ValueFormatter.full

    public init(
        unit: FeeUnit,
        decimals: Int,
        symbol: String,
        formatter: NumericFormatter = NumericFormatter(),
    ) {
        self.unit = unit
        self.decimals = decimals
        self.symbol = symbol
        numericFormatter = formatter
    }

    public var value: String {
        switch unit.type {
        case .satVb: Localized.FeeRate.satvB(unitValueText)
        case .gwei: Localized.FeeRate.gwei(unitValueText)
        case .native: unitValueText
        }
    }

    public var suffix: String {
        switch unit.type {
        case .satVb: Localized.FeeRate.satvB("")
        case .gwei: Localized.FeeRate.gwei("")
        case .native: symbol
        }
    }

    private var unitValueText: String {
        switch unit.type {
        case .satVb:
            return valueFormatter.string(unit.value, decimals: decimals)
        case .gwei:
            guard let value = try? ValueFormatter.full.double(from: unit.value, decimals: decimals) else {
                return ""
            }
            return numericFormatter.string(value)
        case .native:
            return String(
                format: "%@ %@",
                valueFormatter.string(unit.value, decimals: decimals),
                symbol,
            )
        }
    }
}
