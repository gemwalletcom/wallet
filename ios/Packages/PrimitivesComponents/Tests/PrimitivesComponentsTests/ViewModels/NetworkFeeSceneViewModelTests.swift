// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
@testable import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

@MainActor
struct NetworkFeeSceneViewModelTests {
    @Test
    func showFeeRatesSelector() {
        #expect(NetworkFeeSceneViewModel.mock(rates: [.defaultRate()]).showFeeRates == false)
        #expect(NetworkFeeSceneViewModel.mock(rates: [.defaultRate(), .defaultRate()]).showFeeRates)
    }

    @Test
    func showFeeDetailsForLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(
            rates: [.defaultRate()],
            feeAmount: BigInt(1_000_000_000_000_000),
        )

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForMultipleRates() {
        let model = NetworkFeeSceneViewModel.mock(rates: [.defaultRate(), .defaultRate()])

        #expect(model.showFeeRates)
        #expect(model.showFeeDetails)
    }

    @Test
    func hideFeeDetailsWithoutLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(rates: [.defaultRate()])

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails == false)
    }

    @Test
    func valueMatchesSelectedFeeRateEthereumValueText() {
        let model = NetworkFeeSceneViewModel.mock(rates: [.defaultRate()])

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000000001 gwei")
    }

    @Test
    func valueMatchesSelectedFeeRateSolanaValueText() {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            rates: [FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 5000, priorityFee: 100_000))],
        )

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000105 SOL")
    }

    @Test
    func valueMatchesSelectedFeeRateBitcoinValueText() {
        let model = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), rates: [.defaultRate()])

        #expect(model.selectedFeeRateViewModel?.valueText == "0.1 sat/vB")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: .mockSolana(),
            rates: [FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))],
            feeAssetPrice: Price(price: 150.0, priceChangePercentage24h: 0, updatedAt: Date()),
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(
            rates: [.defaultRate()],
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
            rates: [FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))],
            feeAmount: BigInt(5000),
        )
        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func feeAmountScalesProportionallyToSelectedRate() {
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 2, priorityFee: 0)),
            FeeRate(priority: .fast, gasPriceType: .eip1559(gasPrice: 4, priorityFee: 0)),
        ]
        let model = NetworkFeeSceneViewModel.mock(rates: rates, feeAmount: BigInt(1000))

        #expect(model.estimatedFee(for: rates[0]) == BigInt(1000))
        #expect(model.estimatedFee(for: rates[1]) == BigInt(2000))
    }

    @Test
    func valueForRateUsesScaledLoadedFeeForNativeChain() throws {
        let feeAsset = Asset.mockSUI()
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            rates: [
                FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 110)),
                FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 200)),
            ],
            feeAmount: BigInt(110_000),
        )

        let normalRate = try #require(model.feeRatesViewModels.first { $0.feeRate.priority == .normal })
        let fastRate = try #require(model.feeRatesViewModels.first { $0.feeRate.priority == .fast })

        #expect(model.valueForRate(normalRate) == feeAsset.feeText(110_000))
        #expect(model.valueForRate(fastRate) == feeAsset.feeText(200_000))
        #expect(model.valueForRate(normalRate) == model.value)
        #expect(model.valueForRate(fastRate) != fastRate.valueText)
    }

    @Test
    func valueForRateUsesGasPriceRateForNonNativeChains() throws {
        let ethModel = NetworkFeeSceneViewModel.mock(
            rates: [FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 1_000_000_000, priorityFee: 0))],
            feeAmount: BigInt(21_000_000_000_000),
        )
        let ethVM = try #require(ethModel.feeRatesViewModels.first)

        #expect(ethModel.valueForRate(ethVM) == ethVM.valueText)
        #expect(ethModel.valueForRate(ethVM) != ethModel.value)

        let bitcoinModel = NetworkFeeSceneViewModel.mock(feeAsset: .mock(), rates: [.defaultRate()], feeAmount: BigInt(10000))
        let bitcoinVM = try #require(bitcoinModel.feeRatesViewModels.first)

        #expect(bitcoinModel.valueForRate(bitcoinVM) == bitcoinVM.valueText)
        #expect(bitcoinModel.valueForRate(bitcoinVM) != bitcoinModel.value)
    }

    @Test
    func feeAmountReturnsNilWithoutLoadedFee() {
        let model = NetworkFeeSceneViewModel.mock()
        let rate = FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 1, priorityFee: 0))

        #expect(model.estimatedFee(for: rate) == nil)
    }

    @Test
    func selectedRateFollowsSelection() {
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 2)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), rates: rates).selectedFeeRateViewModel?.feeRate.priority == .normal)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), selection: .preset(.fast), rates: rates).selectedFeeRateViewModel?.feeRate.priority == .fast)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), selection: .custom(5), rates: rates).selectedFeeRateViewModel == nil)
    }

    @Test
    func selectForwardsSelectionToOwner() async {
        await confirmation { selected in
            NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), onSelect: {
                #expect($0 == .preset(.fast))
                selected()
            })
            .select(.preset(.fast))
        }
    }

    @Test
    func supportsCustomFeeWhenSelectableWithMultipleRates() {
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 2)),
        ]
        let onSelect: @MainActor (FeeSelection) -> Void = { _ in }

        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), rates: rates, onSelect: onSelect).supportsCustomFee)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mockSolana(), rates: rates, onSelect: onSelect).supportsCustomFee == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), rates: rates).supportsCustomFee == false)
        #expect(NetworkFeeSceneViewModel.mock(feeAsset: .mock(), rates: [rates[0]], onSelect: onSelect).supportsCustomFee == false)
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
                #expect($0 == .custom(40))
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
                #expect($0 == .custom(200))
                selected()
            }).customFeeModel()
            custom.input = "20"
            custom.confirm()
        }

        let reopened = bitcoinScene(selection: .custom(200)).customFeeModel()
        reopened.input = "21"
        #expect(reopened.isConfirmEnabled == false)
        #expect(reopened.errorText != nil)
    }

    @Test
    func customRowShowsValueOnlyWhenSelected() {
        let selected = bitcoinScene(selection: .custom(200))
        #expect(selected.isCustomSelected)
        #expect(selected.customRowItem.subtitle != nil)

        let preset = bitcoinScene()
        #expect(preset.isCustomSelected == false)
        #expect(preset.customRowItem.subtitle == nil)
    }

    private func bitcoinScene(
        selection: FeeSelection = .preset(.normal),
        onSelect: (@MainActor (FeeSelection) -> Void)? = nil,
    ) -> NetworkFeeSceneViewModel {
        .mock(
            feeAsset: .mock(),
            selection: selection,
            rates: [FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 20))],
            feeAmount: BigInt(1000),
            onSelect: onSelect,
        )
    }

    @Test
    func valueUsesFeeAssetForHyperCorePerpetualFee() {
        let feeAmount = BigInt(12_345_678)
        let feeAsset = Asset.hypercoreUSDC()
        let model = NetworkFeeSceneViewModel.mock(
            feeAsset: feeAsset,
            feeAmount: feeAmount,
        )

        #expect(model.value == feeAsset.feeText(feeAmount))
        #expect(model.value != Asset.mockHypercore().feeText(feeAmount))
    }
}

private extension Asset {
    func feeText(_ value: BigInt) -> String {
        AmountDisplay.numeric(asset: self, price: nil, value: value, currency: Currency.usd.rawValue, formatter: .auto).amount.text
    }
}
