// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing
@testable import Transfer

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func loaded() {
        let feeModel = NetworkFeeSceneViewModel.mock(feeRates: .mock([]), feeAssetPrice: .mock(price: 2500), feeAmount: 1_000_000_000_000_000)
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .ready,
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == feeModel.fiatValue)
        #expect(item.subtitle != feeModel.value)
        #expect(selectable == true)
    }

    @Test
    func loadedWithoutFiat() {
        let feeModel = NetworkFeeSceneViewModel.mock(feeRates: .mock([]), feeAmount: 1_000_000_000_000_000)
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .ready,
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(feeModel.fiatValue == nil)
        #expect(item.subtitle == feeModel.value)
        #expect(selectable == true)
    }

    @Test
    func loadedWithSelectableFeeAssetShowsSymbolOnRight() {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())
        let feeModel = NetworkFeeSceneViewModel.mock(
            feeAsset: pathUSD.asset,
            feeAssetPrice: .mock(price: 1),
            feeAmount: 1,
            feeAssets: [pathUSD, usdc],
            onSelectFeeAsset: { _ in },
        )
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .ready,
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, _) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.titleExtra == nil)
        #expect(item.subtitle == feeModel.fiatValue)
        #expect(item.subtitleExtra == pathUSD.asset.symbol)
    }

    @Test
    func error() {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .unavailable,
            feeModel: .mock(
                feeAsset: pathUSD.asset,
                feeAssetPrice: .mock(price: 1),
                feeAmount: 1,
                feeAssets: [pathUSD, usdc],
                onSelectFeeAsset: { _ in },
            ),
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else {
            Issue.record("Expected network fee item")
            return
        }
        #expect(item.subtitle == "-")
        #expect(item.subtitleExtra == nil)
        #expect(selectable == false)
    }

    @Test
    func calculatorError() {
        let feeModel = NetworkFeeSceneViewModel.mock(feeRates: .mock([]), feeAssetPrice: .mock(price: 2500), feeAmount: 1_000_000_000_000_000)
        let model = ConfirmNetworkFeeViewModel(
            feeRow: .ready,
            feeModel: feeModel,
            infoAction: {},
        )

        guard case let .networkFee(item, selectable) = model.itemModel else { return }
        #expect(item.subtitle == feeModel.fiatValue)
        #expect(item.subtitleExtra == nil)
        #expect(selectable == true)
    }
}
