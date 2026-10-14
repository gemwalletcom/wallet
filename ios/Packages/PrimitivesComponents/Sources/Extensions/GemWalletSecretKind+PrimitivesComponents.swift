// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemWalletSecretKind
import GemstonePrimitives

public extension GemWalletSecretKind {
    var securityDocsUrl: URL {
        switch self {
        case .phrase: AppUrl.docs(.howToSecureSecretPhrase)
        case .privateKey: AppUrl.docs(.whatIsPrivateKey)
        }
    }
}
