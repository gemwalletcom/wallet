// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSecurityReminderItem
import Style

extension GemSecurityReminderItem {
    var image: ListItemImageStyle? {
        switch self {
        case .keepSafe: .security(Emoji.WalletAvatar.lock.rawValue)
        case .doNotShare: .security(Emoji.WalletAvatar.warning.rawValue)
        case .noRecovery: .security(Emoji.WalletAvatar.gem.rawValue)
        }
    }
}
