// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import SwiftUI

@Observable
@MainActor
public final class NetworkFeeSceneViewModel {
    private let chain: Chain
    private let currency: Currency

    private var rates: [FeeRate] = []
    private var feeAssetPrice: Price?

    public var priority: FeePriority
    public var value: String?
    public var fiatValue: String?

    public init(
        chain: Chain,
        priority: FeePriority,
        currency: Currency,
        value: String? = nil,
        fiatValue: String? = nil,
    ) {
        self.chain = chain
        self.priority = priority
        self.currency = currency
        self.value = value
        self.fiatValue = fiatValue
    }

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }

    public var feeRatesViewModels: [FeeRateViewModel] {
        rates.map {
            FeeRateViewModel(
                feeRate: $0,
                unitType: chain.feeUnitType,
                decimals: chain.asset.decimals.asInt,
                symbol: chain.asset.symbol,
            )
        }.sorted()
    }

    public var selectedFeeRateViewModel: FeeRateViewModel? {
        feeRatesViewModels.first(where: { $0.feeRate.priority == priority })
    }

    public var showFeeRates: Bool {
        rates.count > 1
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        guard chain.feeUnitType == .native, let price = feeAssetPrice else { return nil }
        let display = AmountDisplay.numeric(
            asset: chain.asset,
            price: price,
            value: rate.feeRate.gasPriceType.totalFee,
            currency: currency.rawValue,
            formatter: .medium,
        )
        return display.fiat?.text
    }
}

// MARK: - Business Logic

public extension NetworkFeeSceneViewModel {
    func update(rates: [FeeRate], feeAssetPrice: Price?) {
        self.rates = rates
        self.feeAssetPrice = feeAssetPrice
    }

    func update(value: String?, fiatValue: String?) {
        self.value = value
        self.fiatValue = fiatValue
    }

    func reset() {
        value = nil
        fiatValue = nil
    }
}
