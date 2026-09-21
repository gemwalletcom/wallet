// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualPositionRow
import func Gemstone.perpetualPositionRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualPositionItemViewModel: ListAssetItemViewable {
    let data: PerpetualPositionData
    let showBalancePrivacy: Binding<Bool>
    var action: ((ListAssetItemAction) -> Void)?

    private let row: GemPerpetualPositionRow

    init(
        data: PerpetualPositionData,
        showBalancePrivacy: Binding<Bool> = .constant(false),
    ) {
        self.data = data
        self.showBalancePrivacy = showBalancePrivacy
        row = perpetualPositionRow(perpetual: data.perpetual.toGem(), asset: data.asset.toGem(), position: data.position.toGem())
    }

    var name: String {
        row.title
    }

    var symbol: String? {
        nil
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: data.perpetual.assetId).assetImage
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

extension PerpetualPositionItemViewModel: Identifiable {
    var id: String {
        data.position.id
    }
}
