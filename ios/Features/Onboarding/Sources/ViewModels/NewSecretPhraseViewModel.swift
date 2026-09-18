import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI
import func Gemstone.secretPhraseCopy

struct NewSecretPhraseViewModel: SecretPhraseViewableModel {
    private let onCreateWallet: ([String]) -> Void
    let words: [String]

    var calloutViewStyle: CalloutViewStyle? {
        .header(title: Localized.SecretPhrase.savePhraseSafely)
    }

    var continueAction: VoidAction {
        { onCreateWallet(words) }
    }

    init(
        words: [String],
        onCreateWallet: @escaping (([String]) -> Void),
    ) {
        self.words = words
        self.onCreateWallet = onCreateWallet
    }

    var title: String {
        Localized.Wallet.New.title
    }

    var type: SecretPhraseDataType {
        .words(rows: SecretPhraseRow.rows(for: words))
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: secretPhraseCopy(words: words))
    }
}
