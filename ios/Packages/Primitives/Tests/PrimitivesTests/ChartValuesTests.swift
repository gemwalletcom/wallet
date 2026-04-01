// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives
import PrimitivesTestKit
import Testing

struct ChartValuesTests {
    @Test
    func chartValues() throws {
        let values = ChartValues.mock(values: [100, 150, 80, 120])

        #expect(values.lowerBoundValue == 80)
        #expect(values.upperBoundValue == 150)
        #expect(values.yScale == [76.5, 153.5])
        #expect(values.hasVariation == true)
        #expect(values.firstValue == 100)
        #expect(values.lastValue == 120)
        #expect(values.baseValue == 100)

        #expect(throws: Error.self) { _ = try ChartValues.from(charts: []) }
        #expect(ChartValues.mock(values: [100, 100]).hasVariation == false)
    }
}
