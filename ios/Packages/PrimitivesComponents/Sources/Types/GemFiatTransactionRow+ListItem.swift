// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemFiatTransactionRow
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

extension GemFiatTransactionRow: @retroactive Identifiable {}

public extension GemFiatTransactionRow {
    var listItemModel: ListItemModel {
        ListItemModel(
            title: quoteType.toPrimitives().action,
            titleTag: badge?.text,
            titleTagStyle: badge?.textStyle ?? ListItemModel.StyleDefaults.titleTagStyle,
            titleExtra: subtitle,
            titleStyleExtra: .footnote,
            subtitle: value.text(),
            subtitleStyle: TextStyle(font: .callout, color: value.tone.color),
            subtitleExtra: fiatValue.text(),
            subtitleStyleExtra: TextStyle(font: .footnote, color: Colors.gray),
            imageStyle: .asset(assetImage: .image(provider.toPrimitives().image)),
        )
    }

    var detailsURL: URL? {
        detailsUrl?.asURL
    }
}
