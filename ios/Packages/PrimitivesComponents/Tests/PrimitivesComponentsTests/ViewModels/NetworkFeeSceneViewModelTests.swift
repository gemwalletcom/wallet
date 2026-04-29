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

        #expect(model.selectedFeeRateViewModel?.valueText == "1 sat/vB")
    }

    @Test
    func fiatValueForNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))
        let price = Price(price: 150.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [rate], feeAssetPrice: price)

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueNilForNonNativeFeeType() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let price = Price(price: 3000.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [.defaultRate()], feeAssetPrice: price)

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() throws {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))

        model.update(rates: [rate], feeAssetPrice: nil)

        let feeRateVM = try #require(model.feeRatesViewModels.first)

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func feeAmountScalesProportionallyToSelectedRate() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let rates = [
            FeeRate(priority: .slow, gasPriceType: .eip1559(gasPrice: 1, priorityFee: 0)),
            FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 2, priorityFee: 0)),
            FeeRate(priority: .fast, gasPriceType: .eip1559(gasPrice: 4, priorityFee: 0)),
        ]
        model.update(rates: rates, feeAssetPrice: nil)
        model.update(feeAmount: BigInt(1000))

        #expect(model.feeAmount(for: rates[0]) == BigInt(500))
        #expect(model.feeAmount(for: rates[1]) == BigInt(1000))
        #expect(model.feeAmount(for: rates[2]) == BigInt(2000))
    }

    @Test
    func feeAmountReturnsNilWithoutLoadedFee() {
        let model = NetworkFeeSceneViewModel.mock(chain: .ethereum)
        let rate = FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 1, priorityFee: 0))

        #expect(model.feeAmount(for: rate) == nil)
    }

    @Test
    func prioritySelection() {
        let model = NetworkFeeSceneViewModel.mock(chain: .solana)
        let rates = [
            FeeRate(priority: .slow, gasPriceType: .regular(gasPrice: 1)),
            FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: 2)),
            FeeRate(priority: .fast, gasPriceType: .regular(gasPrice: 3)),
        ]

        model.update(rates: rates, feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.feeRate.priority == .normal)

        model.priority = .fast
        #expect(model.selectedFeeRateViewModel?.feeRate.priority == .fast)

        model.priority = .slow
        #expect(model.selectedFeeRateViewModel?.feeRate.priority == .slow)
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

        let expected = AmountDisplay.numeric(
            asset: feeAsset,
            price: nil,
            value: feeAmount,
            currency: Currency.usd.rawValue,
            formatter: .auto,
        ).amount.text
        let wrong = AmountDisplay.numeric(
            asset: .mockHypercore(),
            price: nil,
            value: feeAmount,
            currency: Currency.usd.rawValue,
            formatter: .auto,
        ).amount.text

        #expect(model.value == expected)
        #expect(model.value != wrong)
    }
}
