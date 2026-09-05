// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmFeeSelection
import BigInt
import Components
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkFeeSceneViewModel {
    private let feeAsset: Asset
    private let currency: Currency
    private let selection: GemConfirmFeeSelection
    private let feeRates: GemFeeRateRows?
    private let feeAssetPrice: Price?
    private let feeAmount: BigInt?
    private let feeAssets: [FeeAssetItem]
    private let onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)?
    private let onSelectFeeAsset: (@MainActor (AssetId) -> Void)?

    public init(
        feeAsset: Asset,
        currency: Currency,
        selection: GemConfirmFeeSelection,
        feeRates: GemFeeRateRows? = nil,
        feeAssetPrice: Price? = nil,
        feeAmount: BigInt? = nil,
        feeAssets: [FeeAssetItem] = [],
        onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) {
        self.feeAsset = feeAsset
        self.currency = currency
        self.selection = selection
        self.feeRates = feeRates
        self.feeAssetPrice = feeAssetPrice
        self.feeAmount = feeAmount
        self.feeAssets = feeAssets
        self.onSelect = onSelect
        self.onSelectFeeAsset = onSelectFeeAsset
    }

    // MARK: - Network Fee

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }
    public var value: String? { feeAmount.map { display(for: $0).amount.text } }
    public var fiatValue: String? { feeAmount.flatMap { display(for: $0).fiat?.text } }
    public var showFeeRates: Bool { rows.count > 1 }
    public var showFeeDetails: Bool { showFeeAssets || feeRates != nil }
    public var feeAssetSymbol: String? { showFeeAssets && fiatValue != nil ? feeAsset.symbol : nil }

    var showFeeAssets: Bool {
        onSelectFeeAsset != nil && feeAssets.contains { $0.asset.id != feeAsset.id }
    }

    var selectedFeeAssetItem: FeeAssetItem {
        feeAssets.first(where: { $0.asset.id == feeAsset.id })
            ?? FeeAssetItem(asset: feeAsset, balance: .zero, price: nil, currency: currency, isSelected: false)
    }

    var feeAssetsViewModel: FeeAssetsViewModel {
        FeeAssetsViewModel(
            state: .data(.plain(feeAssets.map { $0.selected($0.asset.id == feeAsset.id) })),
        )
    }

    // MARK: - Fee Rates

    public var feeRatesViewModels: [FeeRateViewModel] {
        rows.map { feeRateViewModel(priority: $0.priority.map(), unitValue: $0.unitValue, fee: $0.fee) }
    }

    public var selectedFeeRateViewModel: FeeRateViewModel? {
        guard let priority = selection.selectedPriority()?.map() else { return nil }
        return feeRatesViewModels.first(where: { $0.priority == priority })
    }

    public func isSelected(_ rate: FeeRateViewModel) -> Bool {
        selection.selectedPriority()?.map() == rate.priority
    }

    public func rowItem(for rate: FeeRateViewModel) -> ListItemModel {
        rowItem(title: rate.title, rate: rate)
    }

    public func valueForRate(_ rate: FeeRateViewModel) -> String {
        switch rate.unitType {
        case .native: rate.fee.map { display(for: $0).amount.text } ?? rate.valueText
        case .gwei, .satVb: rate.valueText
        }
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        rate.fee.flatMap { display(for: $0).fiat?.text }
    }

    // MARK: - Custom Fee

    public var supportsCustomFee: Bool { onSelect != nil && feeRates?.supportsCustomFee == true }
    public var isCustomSelected: Bool { selection.customGasPrice() != nil }
    public var customRowItem: ListItemModel { rowItem(title: Localized.FeeRate.custom, rate: customFeeRateViewModel) }

    @MainActor
    public func customFeeModel() -> NetworkFeeCustomViewModel {
        NetworkFeeCustomViewModel(
            chain: feeAsset.chain,
            feeAsset: feeAsset,
            feeAssetPrice: feeAssetPrice,
            currency: currency,
            unitType: unitType,
            decimals: unitDecimals,
            baseFee: feeAmount,
            baseTotal: feeRates?.selectedTotal,
            normalTotal: feeRates?.normalTotal ?? feeRates?.selectedTotal,
            initialRate: selection.customGasPrice(),
            onSelect: { onSelect?(.custom(gasPrice: $0)) },
        )
    }

    @MainActor
    public func select(_ selection: GemConfirmFeeSelection) {
        onSelect?(selection)
    }

    @MainActor
    func selectFeeAsset(_ item: FeeAssetItem) {
        onSelectFeeAsset?(item.id)
    }
}

// MARK: - Private

private extension NetworkFeeSceneViewModel {
    var rows: [GemFeeRateRow] { feeRates?.rows ?? [] }
    var unitType: FeeUnitType { feeRates?.unitType.map() ?? .native }
    var unitDecimals: Int { feeRates.map { Int($0.unitDecimals) } ?? feeAsset.decimals.asInt }

    func feeRateViewModel(priority: FeePriority, unitValue: BigInt, fee: BigInt?) -> FeeRateViewModel {
        FeeRateViewModel(
            priority: priority,
            unitValue: unitValue,
            fee: fee,
            unitType: unitType,
            decimals: unitDecimals,
            symbol: feeAsset.symbol,
        )
    }

    var customFeeRateViewModel: FeeRateViewModel? {
        selection.customGasPrice().map { feeRateViewModel(priority: .normal, unitValue: $0, fee: feeAmount) }
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
