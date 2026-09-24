// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.fiatTransactionRows
import enum Gemstone.GemFiatTransactionBadge
import struct Gemstone.GemFiatTransactionRow
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct FiatTransactionViewModel: Sendable, Identifiable {
    public let id: String
    public let createdAt: Date
    private let row: GemFiatTransactionRow
    private let locale: Locale

    public init(info: FiatTransactionAssetData, row: GemFiatTransactionRow, locale: Locale = .current) {
        id = info.id
        createdAt = info.createdAt
        self.row = row
        self.locale = locale
    }

    public static func models(_ transactions: [FiatTransactionAssetData], locale: Locale = .current) -> [FiatTransactionViewModel] {
        zip(transactions, fiatTransactionRows(data: transactions.map { $0.toGem() })).map {
            FiatTransactionViewModel(info: $0, row: $1, locale: locale)
        }
    }

    public var listItemModel: ListItemModel {
        ListItemModel(
            title: row.quoteType.toPrimitives().action,
            titleTag: row.badge?.text,
            titleTagStyle: row.badge.map(badgeStyle) ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleExtra: row.subtitle,
            titleStyleExtra: .footnote,
            subtitle: row.value.text(locale: locale),
            subtitleStyle: TextStyle(font: .callout, color: row.value.tone.color),
            subtitleExtra: row.fiatValue.text(locale: locale),
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: .image(row.provider.toPrimitives().image)),
        )
    }

    public var detailsUrl: URL? {
        row.detailsUrl?.asURL
    }
}

// MARK: - Private

extension FiatTransactionViewModel {
    private func badgeStyle(_ badge: GemFiatTransactionBadge) -> TextStyle {
        badge.textStyle
    }
}
