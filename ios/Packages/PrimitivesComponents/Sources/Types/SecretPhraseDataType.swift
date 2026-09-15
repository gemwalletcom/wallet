// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum SecretPhraseDataType {
    case words(rows: [SecretPhraseRow])
    case privateKey(key: String)
}
