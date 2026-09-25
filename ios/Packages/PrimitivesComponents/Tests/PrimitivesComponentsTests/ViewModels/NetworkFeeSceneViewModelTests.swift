// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeRateRows
import struct Gemstone.GemFormattedNumber
import enum Gemstone.GemLocalizedText
import GemstonePrimitives
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents

extension NetworkFeeSceneViewModel {
    var selectedFeeRate: FeeRateViewModel? {
        feeRatesViewModels.first(where: \.isSelected)
    }
}

import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeSceneViewModelTests {
    @Test
    func additionalFeesKeepNetworkFeeTotal() {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeAmount: 1_495_940,
            additionalFees: [(.tokenAccountCreation, 1_488_440), (.tokenAccountCreation, 1000)],
        )

        #expect(model.value == "0.001495 SOL")
        #expect(model.feeItems.map(\.title) == ["Account Activation Fee", "Account Activation Fee"])
        #expect(model.feeItems.first?.subtitle == "0.001488 SOL")
        #expect(NetworkFeeSceneViewModel.mock().feeItems.isEmpty)
    }

    @Test
    func showFeeRatesSelector() {
        #expect(NetworkFeeSceneViewModel.mock(feeRates: .mock([(.normal, 1, nil)])).showFeeRates == false)
        #expect(NetworkFeeSceneViewModel.mock(feeRates: .mock([(.normal, 1, nil), (.fast, 2, nil)])).showFeeRates)
    }

    @Test
    func showFeeAssetsOnlyWhenAlternativeAssetIsSelectable() {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())
        let onSelect: @MainActor (AssetId) -> Void = { _ in }
        let selectable = NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc], showsFeeAssets: true, onSelectFeeAsset: onSelect)

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD], onSelectFeeAsset: onSelect).showFeeAssets == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc], showsFeeAssets: true).showFeeAssets == false)
        #expect(selectable.showFeeAssets)
        #expect(selectable.showFeeDetails)
    }

    @Test
    func feeAssetSymbolShownOnlyWhenSelectable() {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())
        let onSelect: @MainActor (AssetId) -> Void = { _ in }

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset).feeAssetSymbol == nil)
        #expect(NetworkFeeSceneViewModel.mock(
            feeAsset: pathUSD.asset,
            feeAssets: [pathUSD, usdc],
            showsFeeAssets: true,
            onSelectFeeAsset: onSelect,
        ).feeAssetSymbol == nil)
        #expect(NetworkFeeSceneViewModel.mock(
            feeAsset: pathUSD.asset,
            feeAssetPrice: .mock(price: 1),
            feeAmount: 1,
            feeAssets: [pathUSD, usdc],
            showsFeeAssets: true,
            onSelectFeeAsset: onSelect,
        ).feeAssetSymbol == pathUSD.asset.symbol)
    }

    @Test
    func selectFeeAssetForwardsAssetIdToOwner() async {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())

        await confirmation { selected in
            let model = NetworkFeeSceneViewModel.mock(
                feeAsset: pathUSD.asset,
                feeAssets: [pathUSD, usdc],
                onSelectFeeAsset: {
                    #expect($0 == usdc.asset.id)
                    selected()
                },
            )
            model.selectFeeAsset(usdc)
        }
    }

    @Test
    func showFeeDetailsForLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(
            feeRates: .mock([(.normal, 1, nil)]),
            feeAmount: BigInt(1_000_000_000_000_000),
        )

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForMultipleRates() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: .mock([(.normal, 1, nil), (.fast, 2, nil)]))

        #expect(model.showFeeRates)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForSingleRateWhileReloading() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: .mock([(.normal, 1, nil)]))

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func hideFeeDetailsBeforePreload() {
        #expect(NetworkFeeSceneViewModel.mock().showFeeDetails == false)
    }

    @Test
    func aRateRowShowsTheCoreRateWithItsLocalizedUnit() {
        let rate = GemFormattedNumber.mock(value: 2.5, unit: .plain, display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain)
        let solRate = GemFormattedNumber.mock(value: 2.5, unit: .symbol(symbol: "SOL"), display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain)
        let valueText = { (value: GemLocalizedText) in
            NetworkFeeSceneViewModel.mock(feeRates: .mock([(.normal, 1, nil)], value: value)).selectedFeeRate?.valueText
        }

        #expect(valueText(.feeRate(rate: rate, unit: .gwei)) == "2.5 gwei")
        #expect(valueText(.feeRate(rate: rate, unit: .satVb)) == "2.5 sat/vB")
        #expect(valueText(.feeRate(rate: solRate, unit: .native)) == "2.5 SOL")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeRates: .mock([(.normal, 5000, 5000)], unitType: .native, decimals: 9),
            feeAssetPrice: .mock(price: 150.0),
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeRates: .mock([(.normal, 1, 21_000_000_000_000)]),
            feeAssetPrice: .mock(price: 3000.0),
            feeAmount: BigInt(21_000_000_000_000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeRates: .mock([(.normal, 5000, 5000)], unitType: .native, decimals: 9),
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func selectForwardsSelectionToOwner() async {
        await confirmation { selected in
            NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), onSelect: {
                #expect($0 == .priority(priority: .fast))
                selected()
            })
            .select(.priority(priority: .fast))
        }
    }

    @Test
    func supportsCustomFeeOnlyWhenSelectable() {
        let rates = GemFeeRateRows.mock([(.normal, 1, nil), (.fast, 2, nil)], unitType: .satVb, decimals: 1, supportsCustomFee: true)
        let onSelect: @MainActor (GemConfirmFeeSelection) -> Void = { _ in }

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rates, onSelect: onSelect).supportsCustomFee)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rates).supportsCustomFee == false)
    }

    @Test
    func customFeeInputConfirmsEnteredRate() throws {
        let custom = try #require(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000).customFeeModel())

        #expect(custom.isConfirmEnabled == false)
        custom.input = "4"
        #expect(custom.isConfirmEnabled)
    }

    @Test
    func customFeeInputRejectsRateAboveMax() throws {
        let custom = try #require(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000).customFeeModel())
        custom.input = "999"

        #expect(custom.isConfirmEnabled == false)
        #expect(custom.errorText != nil)
    }

    @Test
    func customFeeConfirmForwardsSelectionToOwner() async {
        await confirmation { selected in
            let custom = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000, onSelect: {
                #expect($0 == .custom(gasPrice: 40))
                selected()
            }).customFeeModel()!
            custom.input = "4"
            custom.confirm()
        }
    }

    @Test
    func customFeeRejectedRateDoesNotConfirm() async {
        await confirmation(expectedCount: 0) { selected in
            let custom = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000, onSelect: { _ in selected() }).customFeeModel()!
            custom.input = "999"
            custom.confirm()
        }
    }

    @Test
    func customFeeMaxAnchoredToNormalRate() async throws {
        await confirmation { selected in
            let custom = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000, onSelect: {
                #expect($0 == .custom(gasPrice: 200))
                selected()
            }).customFeeModel()!
            custom.input = "20"
            custom.confirm()
        }

        let reopened = try #require(NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(),
            selection: .custom(gasPrice: 200),
            feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true, selectedTotal: 200),
            feeAmount: 1000,
        ).customFeeModel())
        reopened.input = "21"
        #expect(reopened.isConfirmEnabled == false)
        #expect(reopened.errorText != nil)
    }

    @Test
    func customRowShowsValueOnlyWhenSelected() {
        let customRate = GemFormattedNumber.mock(value: 20, unit: .plain, display: .number(precision: .fraction(min: 0, max: 1)), notation: .plain)
        let selected = NetworkFeeSceneViewModel.mock(
            feeAsset: .mock(),
            selection: .custom(gasPrice: 200),
            feeRates: .mock([(.normal, 20, 1000)], selected: nil, unitType: .satVb, decimals: 1, supportsCustomFee: true, selectedTotal: 200, customRate: .feeRate(rate: customRate, unit: .satVb)),
            feeAmount: 1000,
        )
        #expect(selected.isCustomSelected)
        #expect(selected.customRowItem.subtitle == "20 sat/vB")

        let preset = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: .mock([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true), feeAmount: 1000)
        #expect(preset.isCustomSelected == false)
        #expect(preset.customRowItem.subtitle == nil)
    }

    @Test
    func valueUsesFeeAssetForHyperCorePerpetualFee() {
        let feeAmount = BigInt(12_345_678)
        let feeAsset = Asset.mockHypercoreUSDC()
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            feeAmount: feeAmount,
        )

        #expect(model.value == feeAsset.feeText(feeAmount))
        #expect(model.value != Asset.mockHypercore().feeText(feeAmount))
    }
}

extension Asset {
    func feeText(_ value: BigInt) -> String {
        ValueFormatter.auto.string(value, asset: self)
    }
}
