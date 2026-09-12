// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAssetRow
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct ListAssetItemViewModel: ListAssetItemViewable {
    let assetDataModel: AssetDataViewModel
    let row: GemAssetRow

    public let showBalancePrivacy: Binding<Bool>
    public var action: ((ListAssetItemAction) -> Void)?

    public init(
        showBalancePrivacy: Binding<Bool>,
        assetDataModel: AssetDataViewModel,
        row: GemAssetRow = .wallet,
        action: ((ListAssetItemAction) -> Void)? = nil,
    ) {
        self.showBalancePrivacy = showBalancePrivacy
        self.assetDataModel = assetDataModel
        self.row = row
        self.action = action
    }

    public init(
        showBalancePrivacy: Binding<Bool>,
        assetData: AssetData,
        formatter: ValueFormatter,
        currencyCode: String,
    ) {
        let model = AssetDataViewModel(
            assetData: assetData,
            formatter: formatter,
            currencyCode: currencyCode,
        )
        self.init(
            showBalancePrivacy: showBalancePrivacy,
            assetDataModel: model,
            row: .wallet,
            action: nil,
        )
    }

    public var name: String {
        switch row.title {
        case .asset: assetDataModel.name
        case .canonicalAsset: assetDataModel.asset.id.type == .native ? assetDataModel.asset.chain.asset.name : assetDataModel.name
        case .network: assetDataModel.asset.chain.networkName
        }
    }

    public var symbol: String? {
        guard row.showsSymbol, name != assetDataModel.symbol else { return .none }
        return assetDataModel.symbol
    }

    public var subtitleView: ListAssetItemSubtitleView {
        switch row.subtitle {
        case .price:
            .price(
                price: TextValue(
                    text: assetDataModel.priceAmountText,
                    style: TextStyle(font: .footnote, color: Colors.gray),
                ),
                priceChangePercentage24h: TextValue(
                    text: assetDataModel.priceChangeText,
                    style: TextStyle(font: .footnote, color: assetDataModel.priceChangeTextColor),
                ),
            )
        case .network:
            switch assetDataModel.asset.id.type {
            case .native:
                .none
            case .token:
                .type(
                    TextValue(
                        text: assetDataModel.asset.chain.networkName,
                        style: .calloutSecondary,
                    ),
                )
            }
        }
    }

    public var rightView: ListAssetItemRightView {
        switch row.trailing {
        case .balance:
            .balance(
                balance: TextValue(
                    text: assetDataModel.totalBalanceTextWithSymbol,
                    style: TextStyle(font: .callout, color: assetDataModel.balanceTextColor, fontWeight: .semibold),
                ),
                totalFiat: TextValue(
                    text: assetDataModel.fiatBalanceText,
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
        AssetViewModel(asset: assetDataModel.asset).assetImage
    }
}
