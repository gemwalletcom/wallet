// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSwapSideInteraction
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Swap
import Testing

struct SwapTokenViewModelTests {
    @Test
    func aPlaceholderRowAsksForAnAssetAndShowsNoBalance() {
        let model = SwapTokenViewModel(type: .placeholder, interaction: GemSwapSideInteraction(isAmountEditable: true, isAssetSelectable: true, isBalanceActionEnabled: true))

        #expect(model.availableBalanceText == nil)
        #expect(model.assetImage == nil)
        #expect(model.amountPlaceholder.isEmpty)
        #expect(model.isBalanceDisabled)
        #expect(model.fiatBalance(amount: "1") == nil)
    }

    @Test
    func aSelectedRowPricesWhatWasTyped() {
        let asset = Asset.mock(decimals: 8)
        let assetData = AssetDataViewModel(
            assetData: .mock(asset: asset, price: .mock(price: 50000)),
            currency: .usd,
        )
        let model = SwapTokenViewModel(type: .selected(assetData), interaction: GemSwapSideInteraction(isAmountEditable: true, isAssetSelectable: true, isBalanceActionEnabled: true))

        #expect(model.actionTitle == asset.symbol)
        #expect(model.amountPlaceholder == "0")
        #expect(model.fiatBalance(amount: "0") == nil)
        #expect(model.fiatBalance(amount: "2") == "$100,000.00")
    }
}
