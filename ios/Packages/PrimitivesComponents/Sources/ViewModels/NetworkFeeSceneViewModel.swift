// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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
    public var feeAmount: BigInt?

    public init(
        chain: Chain,
        priority: FeePriority,
        currency: Currency,
        feeAmount: BigInt? = nil,
    ) {
        self.chain = chain
        self.priority = priority
        self.currency = currency
        self.feeAmount = feeAmount
    }

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }

    public var value: String? {
        feeAmount.map { display(for: $0).amount.text }
    }

    public var fiatValue: String? {
        feeAmount.flatMap { display(for: $0).fiat?.text }
    }

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

    public func valueForRate(_ rate: FeeRateViewModel) -> String {
        rate.valueText
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        switch chain.feeUnitType {
        case .native: display(for: rate.feeRate.gasPriceType.totalFee).fiat?.text
        case .gwei, .satVb: nil
        }
    }

    private func fiatText(for rate: FeeRate) -> String? {
        feeAmount(for: rate).flatMap { display(for: $0).fiat?.text }
    }

    func feeAmount(for rate: FeeRate) -> BigInt? {
        guard let feeAmount, let selectedRate = rates.first(where: { $0.priority == priority }) else { return nil }
        let selectedTotal = selectedRate.gasPriceType.totalFee
        guard selectedTotal != .zero else { return nil }
        return feeAmount * rate.gasPriceType.totalFee / selectedTotal
    }
}

// MARK: - Business Logic

public extension NetworkFeeSceneViewModel {
    func update(rates: [FeeRate], feeAssetPrice: Price?) {
        self.rates = rates
        self.feeAssetPrice = feeAssetPrice
    }

    func update(feeAmount: BigInt?) {
        self.feeAmount = feeAmount
    }

    func reset() {
        feeAmount = nil
    }
}

// MARK: - Private

private extension NetworkFeeSceneViewModel {
    func display(for amount: BigInt) -> AmountDisplay {
        AmountDisplay.numeric(
            asset: chain.asset,
            price: feeAssetPrice,
            value: amount,
            currency: currency.rawValue,
            formatter: .auto,
        )
    }
}
