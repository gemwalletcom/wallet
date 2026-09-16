// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemChartData
import struct Gemstone.GemChartHeader
import enum Gemstone.GemChartValueType
import GemstonePrimitives
import Primitives
@testable import PrimitivesComponents

public extension ChartHeaderViewModel {
    static func mock(
        period: ChartPeriod = .day,
        date: Date? = nil,
        header: GemChartHeader = .mock(),
        valueType: GemChartValueType = .price,
    ) -> ChartHeaderViewModel {
        ChartHeaderViewModel(
            period: period,
            date: date,
            header: header,
            valueType: valueType,
        )
    }
}

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
