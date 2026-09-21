// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import func Gemstone.secretPhraseCopy
import Localization
import Primitives
import PrimitivesComponents

struct ShowSecretPhraseViewModel: SecretPhraseViewableModel {
    private let words: [String]
    let continueAction: Primitives.VoidAction = nil

    init(words: [String]) {
        self.words = words
    }

    var calloutViewStyle: CalloutViewStyle? {
        .secretDataWarning()
    }

    var title: String {
        Localized.Common.secretPhrase
    }

    var type: SecretPhraseDataType {
        .words(rows: SecretPhraseRow.rows(for: words))
    }

    var copyModel: CopyTypeViewModel {
        CopyTypeViewModel(content: secretPhraseCopy(words: words))
    }
}
