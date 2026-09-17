// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemFormattedNumber
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct TransactionPnlViewModel {
    private let pnl: GemFormattedNumber?

    init(pnl: GemFormattedNumber?) {
        self.pnl = pnl
    }
}

extension TransactionPnlViewModel: ItemModelProvidable {
    var itemModel: TransactionItemModel {
        guard let pnl else {
            return .empty
        }
        return .pnl(ListItemModel(
            title: Localized.Perpetual.pnl,
            subtitle: pnl.text(),
            subtitleStyle: TextStyle(font: .callout, color: pnl.tone.color),
        ))
    }
}
