// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemPrivateKeyWarningItem
import Style

extension GemPrivateKeyWarningItem {
    var image: ListItemImageStyle? {
        switch self {
        case .password: .emoji(Emoji.WalletAvatar.key.rawValue)
        case .neverShare: .emoji(Emoji.WalletAvatar.noEntry.rawValue)
        case .noRecovery: .emoji(Emoji.WalletAvatar.moneyWings.rawValue)
        }
    }
}
