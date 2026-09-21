import Components
import Formatters
import Foundation
import func Gemstone.secretPhraseCopy
import Localization
import Primitives
import PrimitivesComponents
import SwiftUI

struct NewSecretPhraseViewModel: SecretPhraseViewableModel {
    private let onContinue: VoidAction
    let words: [String]

    var calloutViewStyle: CalloutViewStyle? {
        .header(title: Localized.SecretPhrase.savePhraseSafely)
    }

    var continueAction: VoidAction {
        onContinue
    }

    init(
        words: [String],
        onContinue: VoidAction,
    ) {
        self.words = words
        self.onContinue = onContinue
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
