// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import Primitives
import Testing

@testable import PrimitivesComponents

@MainActor
struct NetworkFeeSceneViewModelTests {
    @Test
    func showFeeRatesSelector() {
        let model = NetworkFeeSceneViewModel(chain: .ethereum, priority: .normal, currency: .usd)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)
        #expect(model.showFeeRates == false)

        model.update(rates: [.defaultRate(), .defaultRate()], feeAssetPrice: nil)
        #expect(model.showFeeRates)
    }

    @Test
    func valueMatchesSelectedFeeRateEthereumValueText() {
        let model = NetworkFeeSceneViewModel(chain: .ethereum, priority: .normal, currency: .usd)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000000001 gwei")
    }

    @Test
    func valueMatchesSelectedFeeRateSolanaValueText() {
        let model = NetworkFeeSceneViewModel(chain: .solana, priority: .normal, currency: .usd)

        model.update(rates: [FeeRate(priority: .normal, gasPriceType: .eip1559(gasPrice: 5000, priorityFee: 100_000))], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "0.000105 SOL")
    }

    @Test
    func valueMatchesSelectedFeeRateBitcoinValueText() {
        let model = NetworkFeeSceneViewModel(chain: .bitcoin, priority: .normal, currency: .usd)

        model.update(rates: [.defaultRate()], feeAssetPrice: nil)

        #expect(model.selectedFeeRateViewModel?.valueText == "1 sat/vB")
    }

    @Test
    func fiatValueForNativeFeeType() {
        let model = NetworkFeeSceneViewModel(chain: .solana, priority: .normal, currency: .usd)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))
        let price = Price(price: 150.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [rate], feeAssetPrice: price)

        let feeRateVM = model.feeRatesViewModels.first!

        #expect(model.fiatValueForRate(feeRateVM) != nil)
    }

    @Test
    func fiatValueNilForNonNativeFeeType() {
        let model = NetworkFeeSceneViewModel(chain: .ethereum, priority: .normal, currency: .usd)
        let price = Price(price: 3000.0, priceChangePercentage24h: 0, updatedAt: Date())

        model.update(rates: [.defaultRate()], feeAssetPrice: price)

        let feeRateVM = model.feeRatesViewModels.first!

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func fiatValueNilWithoutPriceData() {
        let model = NetworkFeeSceneViewModel(chain: .solana, priority: .normal, currency: .usd)
        let rate = FeeRate(priority: .normal, gasPriceType: .solana(gasPrice: 5000, priorityFee: 0, unitPrice: 0))

        model.update(rates: [rate], feeAssetPrice: nil)

        let feeRateVM = model.feeRatesViewModels.first!

        #expect(model.fiatValueForRate(feeRateVM) == nil)
    }

    @Test
    func prioritySelection() {
        let model = NetworkFeeSceneViewModel(chain: .solana, priority: .normal, currency: .usd)
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
}
