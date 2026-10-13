// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemSecurityReminderItem
import Style

extension GemSecurityReminderItem {
    var image: ListItemImageStyle? {
        switch self {
        case .keepSafe, .privateKeyKeepSafe: .emoji(Emoji.WalletAvatar.lock.rawValue)
        case .doNotShare, .privateKeyDoNotShare: .emoji(Emoji.WalletAvatar.warning.rawValue)
        case .noRecovery, .privateKeyNoRecovery: .emoji(Emoji.WalletAvatar.gem.rawValue)
        }
    }
}
