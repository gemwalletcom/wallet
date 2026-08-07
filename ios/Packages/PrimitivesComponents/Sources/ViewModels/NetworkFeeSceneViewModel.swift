// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Localization
import Primitives
import Style
import SwiftUI

@Observable
@MainActor
public final class NetworkFeeSceneViewModel {
    public enum Mode: Sendable {
        case standard
        case custom
    }

    private let chain: Chain
    private let feeAsset: Asset
    private let currency: Currency
    private let mode: Mode

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
        mode: Mode = .standard,
    ) {
        self.chain = chain
        self.feeAsset = feeAsset
        selection = .preset(priority)
        self.currency = currency
        self.feeAmount = feeAmount
        self.mode = mode
    }

    // MARK: - Network Fee

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

    public var showFeeRates: Bool {
        rates.count > 1
    }

    public var showFeeDetails: Bool {
        showFeeRates || feeAmount != nil
    }

    // MARK: - Fee Rates

    public var feeRatesViewModels: [FeeRateViewModel] {
        rates.map {
            FeeRateViewModel(
                feeRate: $0,
                unitType: chain.feeUnitType,
                decimals: feeRateDecimals,
                symbol: feeAsset.symbol,
            )
        }.sorted()
    }

    public var selectedFeeRateViewModel: FeeRateViewModel? {
        guard let priority = selection.presetPriority else { return nil }
        return feeRatesViewModels.first(where: { $0.feeRate.priority == priority })
    }

    public func isSelected(_ rate: FeeRateViewModel) -> Bool {
        selection.presetPriority == rate.feeRate.priority
    }

    public func rowItem(for rate: FeeRateViewModel) -> ListItemModel {
        rowItem(title: rate.title, rate: rate)
    }

    public func valueForRate(_ rate: FeeRateViewModel) -> String {
        switch chain.feeUnitType {
        case .native: estimatedFee(for: rate.feeRate).map { display(for: $0).amount.text } ?? rate.valueText
        case .gwei, .satVb: rate.valueText
        }
    }

    private var feeRateDecimals: Int {
        chain.feeRateDecimals(assetDecimals: feeAsset.decimals.asInt)
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        estimatedFee(for: rate.feeRate).flatMap { display(for: $0).fiat?.text }
    }

    func estimatedFee(for rate: FeeRate) -> BigInt? {
        guard let feeAmount, let base = selectedBaseTotalFee, base != .zero else { return nil }
        return feeAmount * rate.gasPriceType.totalFee / base
    }

    // MARK: - Custom Fee

    public var supportsCustomFee: Bool {
        switch mode {
        case .standard: false
        case .custom: chain.customFeeEnabled && showFeeRates
        }
    }

    public var isCustomSelected: Bool {
        selection.customRate != nil
    }

    public var customRowItem: ListItemModel {
        rowItem(title: Localized.FeeRate.custom, rate: customFeeRateViewModel)
    }

    public func customFeeModel() -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: chain,
            feeAsset: feeAsset,
            feeAssetPrice: feeAssetPrice,
            currency: currency,
            baseFee: feeAmount,
            baseTotal: selectedBaseTotalFee,
            normalTotal: (rates.first(where: { $0.priority == .normal }) ?? rates.first)?.gasPriceType.totalFee ?? selectedBaseTotalFee,
            initialRate: selection.customRate,
            onSelect: { [weak self] rate in
                self?.selection = .custom(rate)
            },
        )
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

    func reset() {
        feeAmount = nil
    }
}

// MARK: - Private

private extension NetworkFeeSceneViewModel {
    var customFeeRateViewModel: FeeRateViewModel? {
        selection.customRate.map {
            FeeRateViewModel(
                feeRate: FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: $0)),
                unitType: chain.feeUnitType,
                decimals: feeRateDecimals,
                symbol: feeAsset.symbol,
            )
        }
    }

    var selectedBaseTotalFee: BigInt? {
        switch selection {
        case let .preset(priority): rates.first(where: { $0.priority == priority })?.gasPriceType.totalFee
        case let .custom(rate): rate
        }
    }

    func rowItem(title: String, rate: FeeRateViewModel?) -> ListItemModel {
        ListItemModel(
            title: title,
            subtitle: rate.map { valueForRate($0) },
            subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
            subtitleExtra: rate.flatMap { fiatValueForRate($0) },
            subtitleStyleExtra: .init(font: .footnote, color: Colors.gray),
        )
    }

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
