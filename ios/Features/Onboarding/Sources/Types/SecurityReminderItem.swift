// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSecurityReminderItem
import Style

extension GemSecurityReminderItem {
    var image: ListItemImageStyle? {
        switch self {
        case .keepSafe: .emoji(Emoji.WalletAvatar.lock.rawValue)
        case .doNotShare: .emoji(Emoji.WalletAvatar.warning.rawValue)
        case .noRecovery: .emoji(Emoji.WalletAvatar.gem.rawValue)
        }
    }
}
