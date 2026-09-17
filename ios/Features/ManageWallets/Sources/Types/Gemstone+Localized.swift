// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletSecretKind
import Localization

extension GemWalletSecretKind {
    var title: String {
        switch self {
        case .phrase: Localized.Common.secretPhrase
        case .privateKey: Localized.Common.privateKey
        }
    }
}
