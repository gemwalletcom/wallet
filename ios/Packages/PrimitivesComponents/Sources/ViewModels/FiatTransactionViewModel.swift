// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct FiatTransactionViewModel: Sendable {
    private let info: FiatTransactionAssetData

    private let formatter: ValueFormatter

    public init(info: FiatTransactionAssetData, formatter: ValueFormatter = .short) {
        self.info = info
        self.formatter = formatter
    }

    public var listItemModel: ListItemModel {
        let statusModel = FiatTransactionStatusViewModel(status: info.status)
        return ListItemModel(
            title: typeTitle,
            titleStyle: TextStyle(font: Font.system(.body, weight: .medium), color: .primary),
            titleTag: titleTag(statusModel),
            titleTagStyle: titleTagStyle(statusModel),
            titleExtra: "\(info.asset.name) (\(info.provider.displayName))",
            titleStyleExtra: .footnote,
            subtitle: amount,
            subtitleStyle: TextStyle(font: .callout, color: subtitleColor, fontWeight: .semibold),
            subtitleExtra: fiatValueText,
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: providerImage),
        )
    }

    public var detailsUrl: URL? {
        info.detailsUrl?.asURL
    }
}

// MARK: - Private

extension FiatTransactionViewModel {
    private var typeTitle: String {
        switch info.transactionType {
        case .buy: Localized.Wallet.buy
        case .sell: Localized.Wallet.sell
        }
    }

    private var providerImage: AssetImage {
        .image(info.provider.image)
    }

    private func titleTag(_ model: FiatTransactionStatusViewModel) -> String? {
        switch info.status {
        case .complete, .unknown: .none
        case .pending, .failed: model.title
        }
    }

    private func titleTagStyle(_ model: FiatTransactionStatusViewModel) -> TextStyle {
        TextStyle(
            font: Font.system(.footnote, weight: .medium),
            color: model.color,
            background: model.background,
        )
    }

    private var amount: String {
        guard let value = BigInt(info.value) else { return "" }
        return formatter.string(value, decimals: info.asset.decimals.asInt, currency: info.asset.symbol)
    }

    private var fiatValueText: String {
        CurrencyFormatter(currencyCode: info.fiatCurrency).string(info.fiatAmount)
    }

    private var subtitleColor: Color {
        switch info.status {
        case .failed, .unknown: Colors.gray
        case .pending, .complete: Colors.black
        }
    }
}
