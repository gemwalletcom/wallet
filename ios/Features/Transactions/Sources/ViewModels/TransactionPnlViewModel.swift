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
        return .pnl(
            title: Localized.Perpetual.pnl,
            value: pnl.text(),
            color: pnl.tone.color,
        )
    }
}
