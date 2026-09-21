// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import PrimitivesTestKit
import Style
@testable import Swap
@testable import SwapTestKit
import Testing

struct PriceImpactViewModelTests {
    @Test
    func theImpactReadsSignedAndIsTonedByHowBadItIs() {
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 1_005_000_000).priceImpactText == "+0.50%")
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 990_000_000).priceImpactText == "-1.00%")
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 950_000_000).priceImpactText == "-5.00%")
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 700_000_000).priceImpactText == "-30.00%")

        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 1_005_000_000).priceImpactStyle.color == Colors.green)
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 990_000_000).priceImpactStyle.color == Colors.gray)
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 950_000_000).priceImpactStyle.color == Colors.orange)
        #expect(PriceImpactViewModel.mock(fromValue: 1_000_000_000, toValue: 700_000_000).priceImpactStyle.color == Colors.red)
    }

    @Test
    func onlyAHighImpactWarns() {
        #expect(PriceImpactViewModel.mock(fromValue: 100, toValue: 109).showPriceImpactWarning == false)
        #expect(PriceImpactViewModel.mock(fromValue: 100, toValue: 111).showPriceImpactWarning == true)
        #expect(PriceImpactViewModel.mock(fromValue: 100, toValue: 120).showPriceImpactWarning == true)
    }

    @Test
    func theWarningReadsTheLossAsAPlainSize() {
        #expect(PriceImpactViewModel.mock(fromValue: 100, toValue: 109).highImpactWarningDescription == nil)
        let warning = PriceImpactViewModel.mock(fromValue: 100, toValue: 111).highImpactWarningDescription

        #expect(warning?.contains("11.00%") == true)
        #expect(warning?.contains("-") == false, "the warning says how much is lost, not that it is negative")
    }
}
