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
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)
        #expect(model.showFeeRates == false)

        model.update(rates: [.defaultRate(), .defaultRate()], feeAssetPrice: nil)
        #expect(model.showFeeRates)
    }

    @Test
    func showFeeDetailsForLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)
        model.update(feeAmount: BigInt(1_000_000_000_000_000))

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails)
    }

    @Test
    func showFeeDetailsForMultipleRates() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)

        model.update(rates: [.defaultRate(), .defaultRate()], feeAssetPrice: nil)

        #expect(model.showFeeRates)
        #expect(model.showFeeDetails)
    }

    @Test
    func hideFeeDetailsWithoutLoadedSingleRate() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)

        #expect(model.showFeeRates == false)
        #expect(model.showFeeDetails == false)
    }

    @Test
    func valueMatchesSelectedFeeRateEthereumValueText() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000000001 gwei")
    }

    @Test
    func valueMatchesSelectedFeeRateSolanaValueText() {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)

        model.update(rates: [FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 5000, priorityFee: 100_000))], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000105 SOL")
    }

    @Test
    func valueMatchesSelectedFeeRateBitcoinValueText() {
        let model = NetworkFeeSceneViewModel.mock(chain: .bitcoin)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "0.1 sat/vB")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))
        let price = Price(price: 150.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [rate], feeAssetPrice: price)
        model.update(feeAmount: BigInt(5000))

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let price = Price(price: 3000.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [.defaultRate()], feeAssetPrice: price)
        model.update(feeAmount: BigInt(21_000_000_000_000))

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))

        model.update(rates: [rate], feeAssetPrice: nil)
        model.update(feeAmount: BigInt(5000))

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func feeAmountScalesProportionallyToSelectedRate() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 2, priorityFee: 0)),
            FeeRate(priority: .fast, gasPriceType: .eip1559(gasPrice: 4, priorityFee: 0)),
        ]
        model.update(rates: rates, feeAssetPrice: nil)
        model.update(feeAmount: BigInt(1000))

        #expect(model.estimatedFee(for: rates[0]) == BigInt(1000))
        #expect(model.estimatedFee(for: rates[1]) == BigInt(2000))
    }

    @Test
    func valueForRateUsesScaledLoadedFeeForNativeChain() throws {
        let feeAsset = Asset.mockSUI()
        let model = NetworkFeeSceneViewModel.mock(chain: .sui, feeAsset: feeAsset)
        model.update(
            rates: [
                FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 110)),
                FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 200)),
            ],
            feeAssetPrice: nil,
        )
        model.update(feeAmount: BigInt(110_000))

        let normalRate = try #require(model.feeRatesViewModels.first { $0.feeRate.priority == .normal })
        let fastRate = try #require(model.feeRatesViewModels.first { $0.feeRate.priority == .fast })

        #expect(model.valueForRate(normalRate) == feeAsset.feeText(110_000))
        #expect(model.valueForRate(fastRate) == feeAsset.feeText(200_000))
        #expect(model.valueForRate(normalRate) == model.value)
        #expect(model.valueForRate(fastRate) != fastRate.valueText)
    }

    @Test
    func valueForRateUsesGasPriceRateForNonNativeChains() throws {
        let ethModel = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        ethModel.update(rates: [FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 1_000_000_000, priorityFee: 0))], feeAssetPrice: nil)
        ethModel.update(feeAmount: BigInt(21_000_000_000_000))
        let ethVM = try #require(ethModel.feeRatesViewModels.first)

        #expect(ethModel.valueForRate(ethVM) == ethVM.valueText)
        #expect(ethModel.valueForRate(ethVM) != ethModel.value)

        let bitcoinModel = NetworkFeeSceneViewModel.mock(chain: .bitcoin)
        bitcoinModel.update(rates: [.defaultRate()], feeAssetPrice: nil)
        bitcoinModel.update(feeAmount: BigInt(10000))
        let bitcoinVM = try #require(bitcoinModel.feeRatesViewModels.first)

        #expect(bitcoinModel.valueForRate(bitcoinVM) == bitcoinVM.valueText)
        #expect(bitcoinModel.valueForRate(bitcoinVM) != bitcoinModel.value)
    }

    @Test
    func feeAmountReturnsNilWithoutLoadedFee() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let rate = FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 1, priorityFee: 0))

        #expect(model.estimatedFee(for: rate) == nil)
    }

    @Test
    func prioritySelection() {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 2)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        model.update(rates: rates, feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.feeRate.priority == .normal)

        model.select(.preset(.fast))
        #expect(model.selectedFeeRateViewModel?.feeRate.priority == .fast)

        model.select(.custom(5))
        #expect(model.selectedFeeRateViewModel == nil)
    }

    @Test
    func supportsCustomFeeWhenAllowedWithMultipleRates() {
        let rates = [
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 2)),
        ]

        let allowed = NetworkFeeSceneViewModel.mock(chain: .bitcoin, mode: .custom)
        allowed.update(rates: rates, feeAssetPrice: nil)
        #expect(allowed.supportsCustomFee)

        let chainNotAllowed = NetworkFeeSceneViewModel.mock(chain: .solana, mode: .custom)
        chainNotAllowed.update(rates: rates, feeAssetPrice: nil)
        #expect(chainNotAllowed.supportsCustomFee == false)

        let standardMode = NetworkFeeSceneViewModel.mock(chain: .bitcoin, mode: .standard)
        standardMode.update(rates: rates, feeAssetPrice: nil)
        #expect(standardMode.supportsCustomFee == false)

        let singleRate = NetworkFeeSceneViewModel.mock(chain: .bitcoin, mode: .custom)
        singleRate.update(rates: [rates[0]], feeAssetPrice: nil)
        #expect(singleRate.supportsCustomFee == false)
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
    func customFeeConfirmSetsSceneSelection() {
        let scene = bitcoinScene()
        let custom = scene.customFeeModel()
        custom.input = "4"
        custom.confirm()

        #expect(scene.selection == .custom(40))
        #expect(scene.isCustomSelected)
        #expect(scene.customRowItem.subtitle != nil)
    }

    @Test
    func customFeeRejectedRateDoesNotConfirm() {
        let scene = bitcoinScene()
        let custom = scene.customFeeModel()
        custom.input = "999"
        custom.confirm()

        #expect(scene.isCustomSelected == false)
        #expect(scene.selection == .preset(.normal))
    }

    @Test
    func customFeeMaxAnchoredToNormalRate() {
        let scene = bitcoinScene()
        let selected = scene.customFeeModel()
        selected.input = "20"
        selected.confirm()
        #expect(scene.selection == .custom(200))

        let reopened = scene.customFeeModel()
        reopened.input = "21"
        #expect(reopened.isConfirmEnabled == false)
        #expect(reopened.errorText != nil)
    }

    @Test
    func customRowShowsValueOnlyWhenSelected() {
        let scene = bitcoinScene()
        let custom = scene.customFeeModel()
        custom.input = "20"
        custom.confirm()
        #expect(scene.isCustomSelected)
        #expect(scene.customRowItem.subtitle != nil)

        scene.select(.preset(.normal))
        #expect(scene.isCustomSelected == false)
        #expect(scene.customRowItem.subtitle == nil)
    }

    private func bitcoinScene() -> NetworkFeeSceneViewModel {
        let model = NetworkFeeSceneViewModel.mock(chain: .bitcoin, mode: .custom)
        model.update(rates: [FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 20))], feeAssetPrice: nil)
        model.update(feeAmount: BigInt(1000))
        return model
    }

    @Test
    func valueUsesFeeAssetForHyperCorePerpetualFee() {
        let feeAmount = BigInt(12_345_678)
        let feeAsset = Asset.hypercoreUSDC()
        let model = NetworkFeeSceneViewModel.mock(
            chain: .hyperCore,
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
