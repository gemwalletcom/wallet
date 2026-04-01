// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import Style
import SwiftUI

struct TransactionAmountHeaderViewModel: HeaderViewModel {
    let display: AmountDisplay

    let isWatchWallet: Bool = false
    let buttons: [HeaderButton] = []

    var assetImage: AssetImage? {
        display.assetImage
    }

    var title: String {
        display.amount.text
    }

    var subtitle: String? {
        display.fiat?.text
    }

    var subtitleColor: Color { Colors.gray }
}
