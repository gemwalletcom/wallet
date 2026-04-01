import Foundation
import Localization

enum WalletImportError: LocalizedError {
    case emptyName // TODO: Remove this case, auto generate name
    case invalidSecretPhrase
    case invalidSecretPhraseWord(word: String)
    case invalidAddress

    var errorDescription: String? {
        switch self {
        case .emptyName:
            "Empty Name"
        case .invalidSecretPhrase:
            Localized.Errors.Import.invalidSecretPhrase
        case let .invalidSecretPhraseWord(word):
            Localized.Errors.Import.invalidSecretPhraseWord(word)
        case .invalidAddress:
            Localized.Errors.invalidAddressName
        }
    }
}
