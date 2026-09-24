// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.assetListRow
import struct Gemstone.GemAssetBalance
import enum Gemstone.GemAssetBalanceScope
import struct Gemstone.GemAssetListRow
import struct Gemstone.GemAssetListRowInput
import struct Gemstone.GemAssetRowStyle
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct ListAssetItemViewModel: ListAssetItemViewable {
    let assetDataModel: AssetDataViewModel
    let rowStyle: GemAssetRowStyle
    private let row: GemAssetListRow

    public let showBalancePrivacy: Binding<Bool>
    public var action: ((ListAssetItemAction) -> Void)?

    public init(
        showBalancePrivacy: Binding<Bool>,
        assetDataModel: AssetDataViewModel,
        rowStyle: GemAssetRowStyle,
        row: GemAssetListRow,
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) {
        self.showBalancePrivacy = showBalancePrivacy
        self.assetDataModel = assetDataModel
        self.rowStyle = rowStyle
        self.row = row
        self.action = action
    }

    public init(
        showBalancePrivacy: Binding<Bool>,
        assetDataModel: AssetDataViewModel,
        rowStyle: GemAssetRowStyle,
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) {
        self.init(
            showBalancePrivacy: showBalancePrivacy,
            assetDataModel: assetDataModel,
            rowStyle: rowStyle,
            row: assetListRow(input: Self.rowInput(assetDataModel, rowStyle: rowStyle)),
            action: action,
        )
    }

    static func rowInput(_ assetDataModel: AssetDataViewModel, rowStyle: GemAssetRowStyle) -> GemAssetListRowInput {
        GemAssetListRowInput(
            asset: assetDataModel.asset.toGem(),
            balance: GemAssetBalance(assetDataModel.assetData.balance, assetId: assetDataModel.asset.id, isActive: assetDataModel.assetData.metadata.isActive),
            scope: .total,
            price: assetDataModel.assetData.price?.price,
            change: assetDataModel.assetData.price?.priceChangePercentage24h,
            currency: assetDataModel.currency.toGem(),
            style: rowStyle,
        )
    }

    public init(
        showBalancePrivacy: Binding<Bool>,
        assetData: AssetData,
        formatter _: ValueFormatter,
        currency: Currency,
        rowStyle: GemAssetRowStyle,
    ) {
        let model = AssetDataViewModel(
            assetData: assetData,
            currency: currency,
        )
        self.init(
            showBalancePrivacy: showBalancePrivacy,
            assetDataModel: model,
            rowStyle: rowStyle,
            action: nil,
        )
    }

    public var name: String {
        row.text.title
    }

    public var symbol: String? {
        row.text.symbol
    }

    public var subtitleView: ListAssetItemSubtitleView {
        switch rowStyle.subtitle {
        case .price:
            .price(
                price: TextValue(
                    text: row.price.price?.text() ?? .empty,
                    style: TextStyle(font: .footnote, color: Colors.gray),
                ),
                priceChangePercentage24h: TextValue(
                    text: row.price.change?.text() ?? .empty,
                    style: TextStyle(font: .footnote, color: row.price.change?.tone.color ?? Colors.gray),
                ),
            )
        case .network:
            row.text.network.map { .type(TextValue(text: $0, style: .calloutSecondary)) } ?? .none
        }
    }

    public var rightView: ListAssetItemRightView {
        switch rowStyle.trailing {
        case .balance:
            .balance(
                balance: TextValue(
                    text: row.amount.text(),
                    style: TextStyle(font: .callout, color: row.amount.tone.color, fontWeight: .semibold),
                ),
                totalFiat: TextValue(
                    text: row.fiat?.text() ?? .empty,
                    style: TextStyle(font: .footnote, color: Colors.gray),
                ),
            )
        case .toggle:
            .toggle(assetDataModel.isEnabled)
        case .copy:
            .copy
        case .none:
            .none
        }
    }

    public var assetImage: AssetImage {
        AssetImage(icon: row.icon)
    }
}
