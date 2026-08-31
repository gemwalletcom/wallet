// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import class Gemstone.GemFeeService
import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkFeeSceneViewModel {
    private let feeAsset: Asset
    private let currency: Currency
    private let selection: FeeSelection
    private let rates: [FeeRate]
    private let feeAssetPrice: Price?
    private let feeAmount: BigInt?
    private let feeAssets: [AssetData]
    private let feeService: GemFeeService
    private let onSelect: (@MainActor (FeeSelection) -> Void)?
    private let onSelectFeeAsset: (@MainActor (AssetId) -> Void)?

    public init(
        feeAsset: Asset,
        currency: Currency,
        selection: FeeSelection,
        rates: [FeeRate] = [],
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = nil,
        feeAssets: [AssetData] = [],
        feeService: GemFeeService,
        onSelect: (@MainActor (FeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) {
        self.feeAsset = feeAsset
        self.currency = currency
        self.selection = selection
        self.rates = rates
        self.feeAssetPrice = feeAssetPrice
        self.feeAmount = feeAmount
        self.feeAssets = feeAssets
        self.feeService = feeService
        self.onSelect = onSelect
        self.onSelectFeeAsset = onSelectFeeAsset
    }

    // MARK: - Network Fee

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }
    public var value: String? { feeAmount.map { display(for: $0).amount.text } }
    public var fiatValue: String? { feeAmount.flatMap { display(for: $0).fiat?.text } }
    public var showFeeRates: Bool { rates.count > 1 }
    public var showFeeDetails: Bool { showFeeRates || showFeeAssets || feeAmount != nil }
    public var feeAssetSymbol: String? { showFeeAssets && fiatValue != nil ? feeAsset.symbol : nil }

    var showFeeAssets: Bool {
        onSelectFeeAsset != nil && feeAssets.contains { $0.asset.id != feeAsset.id }
    }

    var selectedFeeAssetItem: FeeAssetItem {
        feeAssetItem(
            feeAssets.first(where: { $0.asset.id == feeAsset.id }) ?? .with(asset: feeAsset),
            isSelected: false,
        )
    }

    var feeAssetsViewModel: FeeAssetsViewModel {
        FeeAssetsViewModel(
            state: .data(.plain(feeAssets.map { feeAssetItem($0, isSelected: $0.asset.id == feeAsset.id) })),
        )
    }

    // MARK: - Fee Rates

    public var feeRatesViewModels: [FeeRateViewModel] {
        rates.map {
            FeeRateViewModel(
                feeRate: $0,
                unitType: feeAsset.chain.feeUnitType,
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
        switch feeAsset.chain.feeUnitType {
        case .native: estimatedFee(for: rate.feeRate).map { display(for: $0).amount.text } ?? rate.valueText
        case .gwei, .satVb: rate.valueText
        }
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        estimatedFee(for: rate.feeRate).flatMap { display(for: $0).fiat?.text }
    }

    func estimatedFee(for rate: FeeRate) -> BigInt? {
        guard let feeAmount, let base = selectedBaseTotalFee, base != .zero else { return nil }
        return feeAmount * rate.gasPriceType.totalFee / base
    }

    // MARK: - Custom Fee

    public var supportsCustomFee: Bool { onSelect != nil && feeAsset.chain.customFeeEnabled && showFeeRates }
    public var isCustomSelected: Bool { selection.customRate != nil }
    public var customRowItem: ListItemModel { rowItem(title: Localized.FeeRate.custom, rate: customFeeRateViewModel) }

    @MainActor
    public func customFeeModel() -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: feeAsset.chain,
            feeAsset: feeAsset,
            feeAssetPrice: feeAssetPrice,
            currency: currency,
            baseFee: feeAmount,
            baseTotal: selectedBaseTotalFee,
            normalTotal: (rates.first(where: { $0.priority == .normal }) ?? rates.first)?.gasPriceType.totalFee ?? selectedBaseTotalFee,
            initialRate: selection.customRate,
            feeService: feeService,
            onSelect: { onSelect?(.custom($0)) },
        )
    }

    @MainActor
    public func select(_ selection: FeeSelection) {
        onSelect?(selection)
    }

    @MainActor
    func selectFeeAsset(_ item: FeeAssetItem) {
        onSelectFeeAsset?(item.id)
    }
}

// MARK: - Private

private extension NetworkFeeSceneViewModel {
    var feeRateDecimals: Int { feeAsset.chain.feeRateDecimals(assetDecimals: feeAsset.decimals.asInt) }

    var customFeeRateViewModel: FeeRateViewModel? {
        selection.customRate.map {
            FeeRateViewModel(
                feeRate: FeeRate(priority: .normal, gasPriceType: .regular(gasPrice: $0)),
                unitType: feeAsset.chain.feeUnitType,
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

    func feeAssetItem(_ assetData: AssetData, isSelected: Bool) -> FeeAssetItem {
        FeeAssetItem(assetData: assetData, currency: currency, isSelected: isSelected)
    }
}
