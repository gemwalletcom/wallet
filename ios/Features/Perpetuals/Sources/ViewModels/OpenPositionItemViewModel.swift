// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualOpenRow
import func Gemstone.perpetualOpenRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct OpenPositionItemViewModel: ListAssetItemViewable {
    private let data: AutocloseOpenData
    private let row: GemPerpetualOpenRow

    var action: ((ListAssetItemAction) -> Void)?

    init(data: AutocloseOpenData) {
        self.data = data
        row = perpetualOpenRow(direction: data.direction.toGem(), leverage: data.leverage, size: data.size)
    }

    var name: String {
        data.symbol
    }

    var symbol: String? {
        nil
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: data.assetId).assetImage
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
                text: row.size?.text() ?? .empty,
                style: TextStyle(font: .body, color: .primary, fontWeight: .medium),
            ),
            totalFiat: TextValue(
                text: .empty,
                style: TextStyle(font: .footnote, color: Colors.secondaryText),
            ),
        )
    }
}
