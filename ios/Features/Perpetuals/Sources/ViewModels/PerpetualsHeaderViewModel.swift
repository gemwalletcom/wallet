// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemPerpetualBalanceHeader
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct PerpetualsHeaderViewModel {
    let header: GemPerpetualBalanceHeader
}

extension PerpetualsHeaderViewModel: ValueHeaderViewModel {
    var isWatchWallet: Bool {
        header.actions.isWatchOnly
    }

    var title: String {
        header.total.text()
    }

    var assetImage: AssetImage? {
        .none
    }

    var subtitle: String? {
        Localized.Wallet.availableBalance(header.available.text())
    }

    var subtitleColor: Color {
        Colors.gray
    }

    var buttons: [HeaderButton] {
        header.actions.headerButtons
    }
}
