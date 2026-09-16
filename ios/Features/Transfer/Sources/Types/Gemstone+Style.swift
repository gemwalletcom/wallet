// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemRecipientSection
import Style
import SwiftUI

extension GemRecipientSection {
    var image: Image {
        switch self {
        case .pinned: Images.System.pin
        case .contacts: Images.System.person
        case .wallets: Images.System.wallet
        case .viewWallets: Images.System.eye
        }
    }
}
