// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Localization
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
public final class NetworkFeeSceneViewModel {
    private let chain: Chain
    private let feeAsset: Asset
    private let currency: Currency
    private let allowsCustomFee: Bool

    private var rates: [FeeRate] = []
    private var feeAssetPrice: Price?

    public var selection: FeeSelection
    public var feeAmount: BigInt?

    public init(
        chain: Chain,
        feeAsset: Asset,
        priority: FeePriority,
        currency: Currency,
        feeAmount: BigInt? = nil,
        allowsCustomFee: Bool = false,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        selection = .preset(priority)
        self.currency = currency
        self.feeAmount = feeAmount
        self.allowsCustomFee = allowsCustomFee
    }

    public var title: String {
        Localized.Transfer.networkFee
    }

    public var infoIcon: String {
        Localized.FeeRates.info
    }

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
                decimals: feeAsset.decimals.asInt,
                symbol: feeAsset.symbol,
            )
        }.sorted()
    }

    public var selectedFeeRateViewModel: FeeRateViewModel? {
        guard let priority = selection.presetPriority else { return nil }
        return feeRatesViewModels.first(where: { $0.feeRate.priority == priority })
    }

    public var showFeeRates: Bool {
        rates.count > 1
    }

    public var showFeeDetails: Bool {
        showFeeRates || feeAmount != nil
    }

    public var supportsCustomFee: Bool {
        guard allowsCustomFee else { return false }
        return switch chain.type {
        case .bitcoin: true
        default: false
        }
    }

    public func isSelected(_ rate: FeeRateViewModel) -> Bool {
        selection.presetPriority == rate.feeRate.priority
    }

    public var isCustomSelected: Bool {
        selection.presetPriority == nil
    }

    public var customFeeEmoji: String {
        Emoji.FeeRate.custom.rawValue
    }

    public var customFeeTitle: String {
        Localized.FeeRate.custom
    }

    public var customRateText: String? {
        guard case let .custom(value) = selection else { return nil }
        return FeeUnitViewModel(
            unit: FeeUnit(type: chain.feeUnitType, value: value),
            decimals: chain.feeUnitDecimals,
            symbol: feeAsset.symbol,
        ).value
    }

    public func valueForRate(_ rate: FeeRateViewModel) -> String {
        switch chain.feeUnitType {
        case .native: feeAmount(for: rate.feeRate).map { display(for: $0).amount.text } ?? rate.valueText
        case .gwei, .satVb: rate.valueText
        }
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        feeAmount(for: rate.feeRate).flatMap { display(for: $0).fiat?.text }
    }

    public func customFeeModel(onComplete: @escaping () -> Void) -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: chain,
            feeAsset: feeAsset,
            feeAssetPrice: feeAssetPrice,
            currency: currency,
            baseFee: feeAmount,
            baseTotal: selectedBaseTotalFee,
            initialRate: selection.customValue,
            onSelect: { [weak self] rate in
                self?.selection = .custom(rate)
                onComplete()
            },
        )
    }

    func feeAmount(for rate: FeeRate) -> BigInt? {
        guard let feeAmount, let base = selectedBaseTotalFee, base != .zero else { return nil }
        return feeAmount * rate.gasPriceType.totalFee / base
    }

    private var selectedBaseTotalFee: BigInt? {
        switch selection {
        case let .preset(priority):
            rates.first(where: { $0.priority == priority })?.gasPriceType.totalFee
        case let .custom(value):
            value
        }
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

    func select(_ selection: FeeSelection) {
        self.selection = selection
    }

    func onSelectPreset(_ rate: FeeRateViewModel) {
        selection = .preset(rate.feeRate.priority)
    }

    func reset() {
        feeAmount = nil
    }
}

// MARK: - Private

private extension NetworkFeeSceneViewModel {
    func display(for amount: BigInt) -> AmountDisplay {
        AmountDisplay.numeric(
            asset: feeAsset,
            price: feeAssetPrice,
            value: amount,
            currency: currency.rawValue,
            formatter: .auto,
        )
    }
}
