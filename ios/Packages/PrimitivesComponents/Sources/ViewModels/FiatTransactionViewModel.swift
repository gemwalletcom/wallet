// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import func Gemstone.fiatTransactionStatus
import enum Gemstone.GemFiatTransactionBadge
import GemstonePrimitives
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
        let status = fiatTransactionStatus(status: info.status.map())
        return ListItemModel(
            title: typeTitle,
            titleStyle: TextStyle(font: Font.system(.body, weight: .medium), color: .primary),
            titleTag: status.badge.map(badgeTitle),
            titleTagStyle: status.badge.map(badgeStyle) ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleExtra: "\(info.asset.name) (\(info.provider.displayName))",
            titleStyleExtra: .footnote,
            subtitle: amount,
            subtitleStyle: TextStyle(font: .callout, color: status.isDimmed ? Colors.gray : Colors.black, fontWeight: .semibold),
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

    private func badgeTitle(_ badge: GemFiatTransactionBadge) -> String {
        switch badge {
        case .pending: Localized.Transaction.Status.pending
        case .failed: Localized.Transaction.Status.failed
        }
    }

    private func badgeColor(_ badge: GemFiatTransactionBadge) -> Color {
        switch badge {
        case .pending: Colors.orange
        case .failed: Colors.red
        }
    }

    private func badgeStyle(_ badge: GemFiatTransactionBadge) -> TextStyle {
        let color = badgeColor(badge)
        return TextStyle(font: Font.system(.footnote, weight: .medium), color: color, background: color.opacity(.light))
    }

    private var amount: String {
        guard let value = BigInt(info.value) else { return "" }
        return formatter.string(value, decimals: info.asset.decimals.asInt, currency: info.asset.symbol)
    }

    private var fiatValueText: String {
        CurrencyFormatter(currencyCode: info.fiatCurrency).string(info.fiatAmount)
    }
}
