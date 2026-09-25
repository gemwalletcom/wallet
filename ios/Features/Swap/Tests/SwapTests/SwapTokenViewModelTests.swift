// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemFormattedNumber
import struct Gemstone.GemSwapSideInteraction
import struct Gemstone.GemSwapSideState
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Swap
import Testing

struct SwapTokenViewModelTests {
    @Test
    func aPlaceholderRowAsksForAnAssetAndShowsNoBalance() {
        let model = SwapTokenViewModel(asset: nil, side: side(isBalanceActionEnabled: false, fiat: nil))

        #expect(model.availableBalanceText == nil)
        #expect(model.assetImage == nil)
        #expect(model.amountPlaceholder.isEmpty)
        #expect(model.isBalanceDisabled)
        #expect(model.fiatText == nil)
    }

    @Test
    func aSelectedRowShowsTheValueCoreFormatted() {
        let asset = Asset.mock(decimals: 8)
        let model = SwapTokenViewModel(
            asset: asset,
            side: side(isBalanceActionEnabled: true, fiat: .mock(value: 100_000, unit: .currency(code: "USD"), display: .number(precision: .fraction(min: 2, max: 2)), notation: .plain, tone: .plain, rounding: .toNearest)),
        )

        #expect(model.actionTitle == asset.symbol)
        #expect(model.amountPlaceholder == "0")
        #expect(model.isBalanceDisabled == false)
        #expect(model.fiatText == "$100,000.00")
    }

    private func side(isBalanceActionEnabled: Bool, fiat: GemFormattedNumber?) -> GemSwapSideState {
        GemSwapSideState(
            interaction: GemSwapSideInteraction(isAmountEditable: true, isAssetSelectable: true, isBalanceActionEnabled: isBalanceActionEnabled),
            balance: nil,
            fiat: fiat,
        )
    }
}
