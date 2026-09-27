// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeRateRow
import struct Gemstone.GemNetworkFeeScreen
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkFeeSceneViewModel {
    private let screen: GemNetworkFeeScreen?
    private let onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)?
    private let onSelectFeeAsset: (@MainActor (AssetId) -> Void)?

    public init(
        screen: GemNetworkFeeScreen?,
        onSelect: (@MainActor (GemConfirmFeeSelection) -> Void)? = nil,
        onSelectFeeAsset: (@MainActor (AssetId) -> Void)? = nil,
    ) {
        self.screen = screen
        self.onSelect = onSelect
        self.onSelectFeeAsset = onSelectFeeAsset
    }

    // MARK: - Network Fee

    public var feeListItem: ListItemModel {
        ListItemModel(title: title, subtitle: value, subtitleExtra: fiatValue, placeholders: [.subtitle])
    }

    public var title: String { Localized.Transfer.networkFee }
    public var infoIcon: String { Localized.FeeRates.info }
    public var value: String? { screen?.fee?.amount.text() }
    public var fiatValue: String? { screen?.fee?.fiat?.text() }
    public var showFeeRates: Bool { screen?.rates?.showsOptions ?? false }

    var feeItems: [ListItemModel] {
        (screen?.additionalFees ?? []).map { item in
            ListItemModel(title: item.option.title, subtitle: item.amount.amount.text(), subtitleExtra: item.amount.fiat?.text())
        }
    }

    var showFeeAssets: Bool {
        onSelectFeeAsset != nil && screen?.feeAsset != nil
    }

    var selectedFeeAssetItem: FeeAssetItem? {
        screen?.feeAsset?.feeAssetItem
    }

    var feeAssetsViewModel: FeeAssetsViewModel {
        FeeAssetsViewModel(
            state: .data(.plain((screen?.feeAssets ?? []).map { $0.feeAssetItem.selected($0.asset.id == screen?.feeAsset?.asset.id) })),
        )
    }

    // MARK: - Fee Rates

    public var feeRateRows: [GemFeeRateRow] {
        screen?.rates?.rows ?? []
    }

    public func rowItem(for row: GemFeeRateRow) -> ListItemModel {
        ListItemModel(
            title: row.title.text,
            subtitle: row.value?.text,
            subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
            subtitleExtra: row.amount?.fiat?.text(),
            subtitleStyleExtra: .init(font: .footnote, color: Colors.gray),
        )
    }

    // MARK: - Custom Fee

    @MainActor
    public func customFeeModel() -> NetworkFeeCustomViewModel? {
        screen?.custom.map { session in
            NetworkFeeCustomViewModel(session: session, onSelect: { onSelect?(.custom(gasPrice: $0)) })
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
