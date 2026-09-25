// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.FeePriority
import enum Gemstone.FeeUnitType
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import enum Gemstone.GemLocalizedText

public extension GemFeeRateRows {
    static func mock(
        _ rows: [(FeePriority, BigInt, BigInt?)],
        selected: FeePriority? = .normal,
        unitType: FeeUnitType = .gwei,
        decimals: UInt32 = 9,
        supportsCustomFee: Bool = false,
        selectedTotal: BigInt? = nil,
        value: GemLocalizedText = .feeRate(rate: .mock(unit: .plain, notation: .plain), unit: .gwei),
        customRate: GemLocalizedText? = nil,
    ) -> GemFeeRateRows {
        GemFeeRateRows(
            rows: rows.map { GemFeeRateRow(priority: $0.0, fee: $0.2, amount: nil, value: value, isSelected: $0.0 == selected) },
            showsOptions: rows.count > 1,
            unitType: unitType,
            unitDecimals: decimals,
            supportsCustomFee: supportsCustomFee,
            selectedTotal: selectedTotal ?? rows.first?.1,
            normalTotal: rows.first?.1,
            customRate: customRate,
        )
    }
}
