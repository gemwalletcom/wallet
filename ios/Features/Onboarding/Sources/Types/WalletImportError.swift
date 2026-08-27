// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemWalletImportError
import Foundation
import Localization

extension GemWalletImportError: @retroactive LocalizedError {
    public var errorDescription: String? {
        switch self {
        case .InvalidSecretPhrase:
            Localized.Errors.Import.invalidSecretPhrase
        case let .InvalidSecretPhraseWords(words):
            Localized.Errors.Import.invalidSecretPhraseWord(words.joined(separator: ", "))
        case .InvalidPrivateKey:
            Localized.Errors.Import.invalidPrivateKey
        case .InvalidAddress:
            Localized.Errors.invalidAddressName
        }
    }
}
