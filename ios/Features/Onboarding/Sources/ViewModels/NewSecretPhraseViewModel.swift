import Components
import Formatters
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

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
        .words(words: WordIndex.rows(for: words))
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(
            type: .secretPhrase,
            copyValue: MnemonicFormatter.fromArray(words: words),
        )
    }
}
