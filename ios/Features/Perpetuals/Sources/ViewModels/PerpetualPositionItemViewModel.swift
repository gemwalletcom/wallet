// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualPositionRow
import func Gemstone.perpetualPositionRows
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualPositionItemViewModel: ListAssetItemViewable, Identifiable {
    let row: GemPerpetualPositionRow
    let showBalancePrivacy: Binding<Bool>
    var action: ((ListAssetItemAction) -> Void)?

    init(row: GemPerpetualPositionRow, showBalancePrivacy: Binding<Bool> = .constant(false)) {
        self.row = row
        self.showBalancePrivacy = showBalancePrivacy
    }

    static func items(_ positions: [PerpetualPositionData], showBalancePrivacy: Binding<Bool>) -> [(data: PerpetualPositionData, model: PerpetualPositionItemViewModel)] {
        zip(positions, perpetualPositionRows(positions: positions.map { $0.toGem() })).map { data, row in
            (data, PerpetualPositionItemViewModel(row: row, showBalancePrivacy: showBalancePrivacy))
        }
    }

    var id: String {
        row.id
    }

    var name: String {
        row.title
    }

    var symbol: String? {
        nil
    }

    var assetImage: AssetImage {
        AssetImage(icon: row.icon)
    }

    var subtitleView: ListAssetItemSubtitleView {
        .type(
            TextValue(
                text: row.position.text,
                style: TextStyle(font: .footnote, color: row.directionTone.color),
            ),
        )
    }

    var rightView: ListAssetItemRightView {
        .balance(
            balance: TextValue(
                text: row.margin.text(),
                style: TextStyle(font: .body, color: .primary, fontWeight: .medium),
            ),
            totalFiat: TextValue(
                text: row.pnl.text,
                style: TextStyle(font: .footnote, color: row.pnlTone.color),
            ),
        )
    }
}
