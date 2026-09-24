// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import func Gemstone.feeAmount
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeAmount
import struct Gemstone.GemFeeOptionItem
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemFeeRateRows
import func Gemstone.showsFeeAssets
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
    private let additionalFees: [GemFeeOptionItem]
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
        additionalFees: [GemFeeOptionItem] = [],
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
        self.additionalFees = additionalFees
        self.feeAssets = feeAssets
        self.onSelect = onSelect
        self.onSelectFeeAsset = onSelectFeeAsset
    }

    // MARK: - Network Fee

    public var feeListItem: ListItemModel {
        ListItemModel(title: title, subtitle: value, subtitleExtra: fiatValue, placeholders: [.subtitle])
    }

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }
    public var value: String? { feeAmount.map { display(for: $0).amount.text() } }
    public var fiatValue: String? { feeAmount.flatMap { display(for: $0).fiat?.text() } }
    public var showFeeRates: Bool { feeRates?.showsOptions ?? false }
    public var showFeeDetails: Bool { showFeeAssets || feeRates != nil }
    public var feeAssetSymbol: String? { showFeeAssets && fiatValue != nil ? feeAsset.symbol : nil }

    var feeItems: [ListItemModel] {
        additionalFees.map { item in
            let amount = display(for: item.value)
            return ListItemModel(title: item.option.title, subtitle: amount.amount.text(), subtitleExtra: amount.fiat?.text())
        }
    }

    var showFeeAssets: Bool {
        onSelectFeeAsset != nil && showsFeeAssets(feeAssetIds: feeAssets.map(\.asset.id.identifier), selected: feeAsset.id.identifier)
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
        rows.map { FeeRateViewModel(priority: $0.priority.toPrimitives(), value: $0.value, fee: $0.fee, isSelected: $0.isSelected) }
    }

    public func rowItem(for rate: FeeRateViewModel) -> ListItemModel {
        rowItem(title: rate.title, rate: rate)
    }

    public func fiatValueForRate(_ rate: FeeRateViewModel) -> String? {
        rate.fee.flatMap { display(for: $0).fiat?.text() }
    }

    // MARK: - Custom Fee

    public var supportsCustomFee: Bool { onSelect != nil && feeRates?.supportsCustomFee == true }
    public var isCustomSelected: Bool { feeRates?.customRate != nil }
    public var customRowItem: ListItemModel { rowItem(title: Localized.FeeRate.custom, rate: customFeeRateViewModel) }

    @MainActor
    public func customFeeModel() -> NetworkFeeCustomViewModel? {
        feeRates.map { rows in
            NetworkFeeCustomViewModel(
                chain: feeAsset.chain,
                feeAsset: feeAsset,
                rows: rows,
                baseFee: feeAmount,
                initialRate: selection.customGasPrice(),
                onSelect: { onSelect?(.custom(gasPrice: $0)) },
                display: display(for:),
            )
        }
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

    var customFeeRateViewModel: FeeRateViewModel? {
        feeRates?.customRate.map { FeeRateViewModel(priority: .normal, value: $0, fee: feeAmount, isSelected: isCustomSelected) }
    }

    func rowItem(title: String, rate: FeeRateViewModel?) -> ListItemModel {
        ListItemModel(
            title: title,
            subtitle: rate.map(\.valueText),
            subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
            subtitleExtra: rate.flatMap { fiatValueForRate($0) },
            subtitleStyleExtra: .init(font: .footnote, color: Colors.gray),
        )
    }

    func display(for amount: BigInt) -> GemFeeAmount {
        Gemstone.feeAmount(asset: feeAsset.toGem(), value: amount, price: feeAssetPrice?.price, currency: currency.toGem())
    }
}
