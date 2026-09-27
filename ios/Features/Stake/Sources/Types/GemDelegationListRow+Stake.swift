// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemDelegationListRow
import PrimitivesComponents
import Style

extension GemDelegationListRow {
    static var titleStyle: TextStyle {
        TextStyle(font: .body, color: .primary, fontWeight: .semibold)
    }

    static var fiatStyle: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    var balanceStyle: TextStyle {
        TextStyle(font: .callout, color: balance.tone.color, fontWeight: .semibold)
    }

    var listItem: ListItemModel {
        ListItemModel(
            title: validator.name,
            titleStyle: Self.titleStyle,
            titleExtra: status.state.title,
            titleStyleExtra: TextStyle(font: .callout, color: status.tone.color),
            subtitle: balance.text(),
            subtitleStyle: balanceStyle,
            subtitleExtra: fiat?.text(),
            subtitleStyleExtra: Self.fiatStyle,
            imageStyle: .asset(assetImage: validator.assetImage),
        )
    }
}
