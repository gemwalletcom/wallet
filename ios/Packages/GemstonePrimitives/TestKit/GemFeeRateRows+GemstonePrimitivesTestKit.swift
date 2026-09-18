// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import enum Gemstone.FeePriority
import enum Gemstone.FeeUnitType
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows

public extension GemFeeRateRows {
    static func mock(
        _ rows: [(FeePriority, BigInt, BigInt?)],
        unitType: FeeUnitType = .gwei,
        decimals: UInt32 = 9,
        supportsCustomFee: Bool = false,
        selectedTotal: BigInt? = nil,
    ) -> GemFeeRateRows {
        GemFeeRateRows(
            rows: rows.map { GemFeeRateRow(priority: $0.0, unitValue: $0.1, fee: $0.2, displayValue: unitType == .native ? ($0.2 ?? $0.1) : $0.1) },
            showsOptions: rows.count > 1,
            unitType: unitType,
            unitDecimals: decimals,
            supportsCustomFee: supportsCustomFee,
            selectedTotal: selectedTotal ?? rows.first?.1,
            normalTotal: rows.first?.1,
        )
    }
}
