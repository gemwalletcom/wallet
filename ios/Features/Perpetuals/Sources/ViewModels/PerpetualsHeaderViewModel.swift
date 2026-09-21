// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemHeaderActions
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
        header.actions == .watchOnly
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
        switch header.actions {
        case .watchOnly: []
        case let .buttons(buttons): buttons.map { HeaderButton(type: $0.kind, isEnabled: $0.isEnabled) }
        }
    }
}
