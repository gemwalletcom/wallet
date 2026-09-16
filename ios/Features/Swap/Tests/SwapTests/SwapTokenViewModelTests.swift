// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
@testable import Swap
import Testing

struct SwapTokenViewModelTests {
    @Test
    func theReceiveRowNeverTakesAnAmountAndThePayRowFollowsTheScene() {
        #expect(SwapTokenInteraction.pay(isEnabled: true).isAmountEditable)
        #expect(SwapTokenInteraction.pay(isEnabled: false).isAmountEditable == false)
        #expect(SwapTokenInteraction.receive(isEnabled: true).isAmountEditable == false)
        #expect(SwapTokenInteraction.receive(isEnabled: true).isAssetSelectable)
        #expect(SwapTokenInteraction.receive(isEnabled: true).isBalanceActionEnabled == false)
    }

    @Test
    func aPlaceholderRowAsksForAnAssetAndShowsNoBalance() {
        let model = SwapTokenViewModel(type: .placeholder, interaction: .pay(isEnabled: true))

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
            assetData: .mock(asset: asset, price: Price(price: 50_000, priceChangePercentage24h: 0, updatedAt: .now)),
            formatter: .short,
            currency: .usd,
        )
        let model = SwapTokenViewModel(type: .selected(assetData), interaction: .pay(isEnabled: true))

        #expect(model.actionTitle == asset.symbol)
        #expect(model.amountPlaceholder == "0")
        #expect(model.fiatBalance(amount: "0") == nil)
        #expect(model.fiatBalance(amount: "2") == "$100,000.00")
    }
}
