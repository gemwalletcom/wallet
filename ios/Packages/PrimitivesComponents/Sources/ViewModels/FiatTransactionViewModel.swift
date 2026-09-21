// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import func Gemstone.fiatTransactionRow
import enum Gemstone.GemFiatTransactionBadge
import struct Gemstone.GemFiatTransactionRow
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct FiatTransactionViewModel: Sendable {
    private let row: GemFiatTransactionRow
    private let locale: Locale

    public init(info: FiatTransactionAssetData, locale: Locale = .current) {
        row = fiatTransactionRow(data: info.toGem())
        self.locale = locale
    }

    public var listItemModel: ListItemModel {
        ListItemModel(
            title: row.quoteType.toPrimitives().action,
            titleTag: row.badge?.text,
            titleTagStyle: row.badge.map(badgeStyle) ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleExtra: row.subtitle,
            titleStyleExtra: .footnote,
            subtitle: row.value.text(locale: locale),
            subtitleStyle: TextStyle(font: .callout, color: row.isDimmed ? Colors.gray : Colors.black),
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
