// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemDelegationListRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct DelegationViewModel: Sendable {
    private let row: GemDelegationListRow

    init(row: GemDelegationListRow) {
        self.row = row
    }

    public var listItem: ListItemModel {
        ListItemModel(
            title: validatorText,
            titleStyle: titleStyle,
            titleExtra: row.status.state.title,
            titleStyleExtra: TextStyle(font: .callout, color: row.status.tone.color),
            subtitle: balanceText,
            subtitleStyle: subtitleStyle,
            subtitleExtra: fiatValueText,
            subtitleStyleExtra: subtitleExtraStyle,
            imageStyle: .asset(assetImage: validatorImage),
        )
    }

    public var titleStyle: TextStyle {
        TextStyle(font: .body, color: .primary, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: row.balance.tone.color, fontWeight: .semibold)
    }

    public var subtitleExtraStyle: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var balanceText: String {
        row.balance.text()
    }

    public var fiatValueText: String? {
        row.fiat?.text()
    }

    public var validatorText: String {
        ValidatorViewModel(row: row.validator).name
    }

    public var validatorImage: AssetImage {
        ValidatorViewModel(row: row.validator).validatorImage
    }
}

// MARK: - Header

public extension DelegationViewModel {
    var header: ValueHeader {
        ValueHeader(assetImage: validatorImage, title: balanceText, subtitle: fiatValueText, subtitleColor: .secondary)
    }
}
