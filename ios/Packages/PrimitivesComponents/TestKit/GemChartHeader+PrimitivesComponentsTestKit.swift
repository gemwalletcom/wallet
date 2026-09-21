// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import GemstonePrimitives
import Primitives

public extension GemChartHeader {
    static func mock(
        value: Double = 100,
        base: Double = 95,
        valueType: GemChartValueType = .price,
        showsSecondaryValue: Bool = false,
    ) -> GemChartHeader {
        GemChartData.mock(
            values: [],
            header: nil,
            valueType: valueType,
            base: base,
            showsSecondaryValue: showsSecondaryValue,
        )
        .headerAt(value: value)
    }
}
