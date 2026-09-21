// Copyright (c). Gem Wallet. All rights reserved.

import Components
@testable import Perpetuals
import Primitives
import PrimitivesTestKit
import Style
import Testing

struct PerpetualPositionViewModelTests {
    @Test
    func leverageText() {
        #expect(PerpetualPositionViewModel(.mock(position: .mock(leverage: 10))).leverageText == "10x")
    }

    @Test
    func directionText() {
        #expect(PerpetualPositionViewModel(.mock(position: .mock(size: 100))).directionText == "Long")
        #expect(PerpetualPositionViewModel(.mock(position: .mock(size: -100))).directionText == "Short")
    }

    @Test
    func positionTypeText() {
        #expect(PerpetualPositionViewModel(.mock(position: .mock(size: 100, leverage: 5))).positionTypeText == "LONG 5x")
    }

    @Test
    func positionTypeColor() {
        #expect(PerpetualPositionViewModel(.mock(position: .mock(direction: .short))).positionTypeColor == Colors.red)
        #expect(PerpetualPositionViewModel(.mock(position: .mock(direction: .long))).positionTypeColor == Colors.green)
    }

//    @Test
//    func liquidationPriceColor() {
//        // Long position: entry $2.00, liquidation $1.50
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 2.00, currencyPrice: 2.00, liquidationPrice: 1.50))).liquidationPriceColor == Colors.secondaryText)
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 2.00, currencyPrice: 1.75, liquidationPrice: 1.50))).liquidationPriceColor == Colors.orange)
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 2.00, currencyPrice: 1.599, liquidationPrice: 1.50))).liquidationPriceColor == Colors.red)
//
//        // Short position: entry $1.27, liquidation $1.91
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 1.27, currencyPrice: 1.27, liquidationPrice: 1.91))).liquidationPriceColor == Colors.secondaryText)
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 1.27, currencyPrice: 1.59, liquidationPrice: 1.91))).liquidationPriceColor == Colors.orange)
//        #expect(PerpetualPositionViewModel(.mock(position: .mock(entryPrice: 1.27, currencyPrice: 1.782, liquidationPrice: 1.91))).liquidationPriceColor == Colors.red)
//    }
}
