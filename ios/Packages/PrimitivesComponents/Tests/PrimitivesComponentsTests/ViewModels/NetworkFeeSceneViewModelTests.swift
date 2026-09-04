// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import enum Gemstone.FeeUnitType
import enum Gemstone.FeePriority
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import enum Gemstone.GemConfirmFeeSelection
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeSceneViewModelTests {
    @Test
    func showFeeRatesSelector() {
        #expect(NetworkFeeSceneViewModel.mock(feeRates: rows([(.normal, 1, nil)])).showFeeRates == false)
        #expect(NetworkFeeSceneViewModel.mock(feeRates: rows([(.normal, 1, nil), (.fast, 2, nil)])).showFeeRates)
    }

    @Test
    func showFeeAssetsOnlyWhenAlternativeAssetIsSelectable() {
        let pathUSD = FeeAssetItem.mock(asset: .mockTempoPathUSD())
        let usdc = FeeAssetItem.mock(asset: .mockTempoUSDC())
        let onSelect: @MainActor (AssetId) -> Void = { _ in }
        let selectable = NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc], onSelectFeeAsset: onSelect)

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD], onSelectFeeAsset: onSelect).showFeeAssets == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [pathUSD, usdc]).showFeeAssets == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: pathUSD.asset, feeAssets: [usdc], onSelectFeeAsset: onSelect).showFeeAssets)
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
            onSelectFeeAsset: onSelect,
        ).feeAssetSymbol == nil)
        #expect(NetworkFeeSceneViewModel.mock(
            feeAsset: pathUSD.asset,
            feeAssetPrice: .mock(price: 1),
            feeAmount: 1,
            feeAssets: [pathUSD, usdc],
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
            feeRates: rows([(.normal, 1, nil)]),
            feeAmount: BigInt(1_000_000_000_000_000),
        )

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForMultipleRates() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: rows([(.normal, 1, nil), (.fast, 2, nil)]))

        #expect(model.showFeeRates)
        #expect(model.showFeeDetails)
    }

    @Test
    func hideFeeDetailsWithoutLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: rows([(.normal, 1, nil)]))

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails == false)
    }

    @Test
    func valueMatchesSelectedFeeRateEthereumValueText() {
        let model = NetworkFeeSceneViewModel.mock(feeRates: rows([(.normal, 1, nil)]))

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000000001 gwei")
    }

    @Test
    func valueMatchesSelectedFeeRateSolanaValueText() {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeRates: rows([(.normal, 105_000, nil)], unitType: .native, decimals: 9),
        )

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000105 SOL")
    }

    @Test
    func valueMatchesSelectedFeeRateBitcoinValueText() {
        let model = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rows([(.normal, 1, nil)], unitType: .satVb, decimals: 1))

        #expect(model.selectedFeeRateViewModel?.valueText == "0.1 sat/vB")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeRates: rows([(.normal, 5000, 5000)], unitType: .native, decimals: 9),
            feeAssetPrice: Price(price: 150.0, priceChangePercentage24h: 0, updatedAt: Date()),
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeRates: rows([(.normal, 1, 21_000_000_000_000)]),
            feeAssetPrice: Price(price: 3000.0, priceChangePercentage24h: 0, updatedAt: Date()),
            feeAmount: BigInt(21_000_000_000_000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            feeRates: rows([(.normal, 5000, 5000)], unitType: .native, decimals: 9),
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func valueForRateUsesScaledLoadedFeeForNativeChain() throws {
        let feeAsset = Asset.mockSUI()
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            feeRates: rows([(.normal, 110, 110_000), (.fast, 200, 200_000)], unitType: .native, decimals: UInt32(feeAsset.decimals)),
            feeAmount: BigInt(110_000),
        )

        let normalRate = try #require(model.feeRatesViewModels.first { $0.priority == .normal })
        let fastRate = try #require(model.feeRatesViewModels.first { $0.priority == .fast })

        #expect(model.valueForRate(normalRate) == feeAsset.feeText(110_000))
        #expect(model.valueForRate(fastRate) == feeAsset.feeText(200_000))
        #expect(model.valueForRate(normalRate) == model.value)
        #expect(model.valueForRate(fastRate) != fastRate.valueText)
    }

    @Test
    func valueForRateUsesGasPriceRateForNonNativeChains() throws {
        let ethModel = NetworkFeeSceneViewModel.mock(
            feeRates: rows([(.normal, 1_000_000_000, 21_000_000_000_000)]),
            feeAmount: BigInt(21_000_000_000_000),
        )
        let ethVM = try #require(ethModel.feeRatesViewModels.first)

        #expect(ethModel.valueForRate(ethVM) == ethVM.valueText)
        #expect(ethModel.valueForRate(ethVM) != ethModel.value)

        let bitcoinModel = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rows([(.normal, 1, 10000)], unitType: .satVb, decimals: 1), feeAmount: BigInt(10000))
        let bitcoinVM = try #require(bitcoinModel.feeRatesViewModels.first)

        #expect(bitcoinModel.valueForRate(bitcoinVM) == bitcoinVM.valueText)
        #expect(bitcoinModel.valueForRate(bitcoinVM) != bitcoinModel.value)
    }

    @Test
    func selectedRateFollowsSelection() {
        let rates = rows([(.normal, 2, nil), (.fast, 3, nil)], unitType: .native, decimals: 9)

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), feeRates: rates).selectedFeeRateViewModel?.priority == .normal)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), selection: .priority(priority: .fast), feeRates: rates).selectedFeeRateViewModel?.priority == .fast)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), selection: .custom(gasPrice: 5), feeRates: rates).selectedFeeRateViewModel == nil)
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
        let rates = rows([(.normal, 1, nil), (.fast, 2, nil)], unitType: .satVb, decimals: 1, supportsCustomFee: true)
        let onSelect: @MainActor (GemConfirmFeeSelection) -> Void = { _ in }

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rates, onSelect: onSelect).supportsCustomFee)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), feeRates: rates).supportsCustomFee == false)
    }

    @Test
    func customFeeInputConfirmsEnteredRate() {
        let custom = bitcoinScene().customFeeModel()

        #expect(custom.isConfirmEnabled == false)
        custom.input = "4"
        #expect(custom.isConfirmEnabled)
    }

    @Test
    func customFeeInputRejectsRateAboveMax() {
        let custom = bitcoinScene().customFeeModel()
        custom.input = "999"

        #expect(custom.isConfirmEnabled == false)
        #expect(custom.errorText != nil)
    }

    @Test
    func customFeeConfirmForwardsSelectionToOwner() async {
        await confirmation { selected in
            let custom = bitcoinScene(onSelect: {
                #expect($0 == .custom(gasPrice: 40))
                selected()
            }).customFeeModel()
            custom.input = "4"
            custom.confirm()
        }
    }

    @Test
    func customFeeRejectedRateDoesNotConfirm() async {
        await confirmation(expectedCount: 0) { selected in
            let custom = bitcoinScene(onSelect: { _ in selected() }).customFeeModel()
            custom.input = "999"
            custom.confirm()
        }
    }

    @Test
    func customFeeMaxAnchoredToNormalRate() async {
        await confirmation { selected in
            let custom = bitcoinScene(onSelect: {
                #expect($0 == .custom(gasPrice: 200))
                selected()
            }).customFeeModel()
            custom.input = "20"
            custom.confirm()
        }

        let reopened = bitcoinScene(selection: .custom(gasPrice: 200)).customFeeModel()
        reopened.input = "21"
        #expect(reopened.isConfirmEnabled == false)
        #expect(reopened.errorText != nil)
    }

    @Test
    func customRowShowsValueOnlyWhenSelected() {
        let selected = bitcoinScene(selection: .custom(gasPrice: 200))
        #expect(selected.isCustomSelected)
        #expect(selected.customRowItem.subtitle != nil)

        let preset = bitcoinScene()
        #expect(preset.isCustomSelected == false)
        #expect(preset.customRowItem.subtitle == nil)
    }

    private func bitcoinScene(
        selection: GemConfirmFeeSelection = .priority(priority: .normal),
        onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        .mock(
            feeAsset: .mock(),
            selection: selection,
            feeRates: rows([(.normal, 20, 1000)], unitType: .satVb, decimals: 1, supportsCustomFee: true, selectedTotal: selection.customGasPrice() ?? 20),
            feeAmount: BigInt(1000),
            onSelect: onSelect,
        )
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

private func rows(
    _ rows: [(Gemstone.FeePriority, BigInt, BigInt?)],
    unitType: Gemstone.FeeUnitType = .gwei,
    decimals: UInt32 = 9,
    supportsCustomFee: Bool = false,
    selectedTotal: BigInt? = nil,
) -> GemFeeRateRows {
    GemFeeRateRows(
        rows: rows.map { GemFeeRateRow(priority: $0.0, unitValue: $0.1, fee: $0.2) },
        unitType: unitType,
        unitDecimals: decimals,
        supportsCustomFee: supportsCustomFee,
        selectedTotal: selectedTotal ?? rows.first?.1,
        normalTotal: rows.first?.1,
    )
}

extension Asset {
    func feeText(_ value: BigInt) -> String {
        AmountDisplay.numeric(asset: self, price: nil, value: value, currency: Currency.usd.rawValue, formatter: .auto).amount.text
    }
}
