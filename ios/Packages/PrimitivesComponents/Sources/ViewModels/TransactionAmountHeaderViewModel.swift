// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import Style
import SwiftUI

struct TransactionAmountHeaderViewModel: ValueHeaderViewModel {
    let display: AmountDisplay
    let fiat: String?

    let isWatchWallet: Bool = false
    let buttons: [HeaderButton] = []

    init(display: AmountDisplay, fiat: String?) {
        self.display = display
        self.fiat = fiat
    }

    init(display: AmountDisplay) {
        self.init(display: display, fiat: display.fiat?.text)
    }

    var assetImage: AssetImage? {
        display.assetImage
    }

    var title: String {
        display.amount.text
    }

    var subtitle: String? {
        fiat
    }

    var subtitleColor: Color {
        Colors.gray
    }
}
