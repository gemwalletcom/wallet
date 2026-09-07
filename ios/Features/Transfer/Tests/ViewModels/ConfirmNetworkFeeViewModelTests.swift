// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemBalanceRequirement
import struct Gemstone.GemConfirmData
import BigInt
import Foundation
@testable import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing
@testable import Transfer
import TransferTestKit

struct ConfirmNetworkFeeViewModelTests {
    @Test
    func loaded() {
        let feeModel = feeModel(feeAssetPrice: Price(price: 2500, priceChangePercentage24h: 0, updatedAt: Date()))
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
        let feeModel = feeModel()
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
        let feeModel = NetworkFeeSceneViewModel(
            feeAsset: pathUSD.asset,
            currency: .usd,
            selection: .priority(priority: .normal),
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
            feeModel: NetworkFeeSceneViewModel(
                feeAsset: pathUSD.asset,
                currency: .usd,
                selection: .priority(priority: .normal),
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
        let feeModel = feeModel(feeAssetPrice: Price(price: 2500, priceChangePercentage24h: 0, updatedAt: Date()))
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

    private func feeModel(
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = BigInt(1_000_000_000_000_000),
    ) -> NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: .mockEthereum(),
            currency: .usd,
            selection: .priority(priority: .normal),
            feeRates: GemConfirmData.mock().feeRateRows(selection: .priority(priority: .normal), feeAsset: Asset.mockEthereum().map()),
            feeAssetPrice: feeAssetPrice,
            feeAmount: feeAmount,
        )
    }
}
