// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletSecretKind

public extension GemWalletSecretKind {
    var securityDocsUrl: URL {
        switch self {
        case .phrase: AppUrl.docs(.howToSecureSecretPhrase)
        case .privateKey: AppUrl.docs(.whatIsPrivateKey)
        }
    }

    var reminderDocsUrl: URL {
        switch self {
        case .phrase: AppUrl.docs(.whatIsSecretPhrase)
        case .privateKey: AppUrl.docs(.whatIsPrivateKey)
        }
    }
}
