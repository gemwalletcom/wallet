// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemRecipientSectionKind
import Style
import SwiftUI

extension GemRecipientSectionKind {
    var image: Image {
        switch self {
        case .pinned: Images.System.pin
        case .contacts: Images.System.person
        case .wallets: Images.System.wallet
        case .viewWallets: Images.System.eye
        }
    }
}
