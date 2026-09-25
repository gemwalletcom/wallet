// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemWalletSecret
import func Gemstone.privateKeyCopy
import func Gemstone.secretPhraseCopy
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

struct SecretDataViewModel {
    private let secret: GemWalletSecret
    let continueAction: VoidAction

    init(secret: GemWalletSecret, continueAction: VoidAction = nil) {
        self.secret = secret
        self.continueAction = continueAction
    }

    var title: String {
        switch secret {
        case .words: continueAction == nil ? Localized.Common.secretPhrase : Localized.Wallet.New.title
        case .privateKey: Localized.Common.privateKey
        }
    }

    var calloutViewStyle: CalloutViewStyle {
        continueAction == nil ? .secretDataWarning() : .header(title: Localized.SecretPhrase.savePhraseSafely)
    }

    var type: SecretPhraseDataType {
        switch secret {
        case let .words(words): .words(rows: SecretPhraseRow.rows(for: words))
        case let .privateKey(key): .privateKey(key: key)
        }
    }

    var copyModel: CopyTypeViewModel {
        switch secret {
        case let .words(words): CopyTypeViewModel(content: secretPhraseCopy(words: words))
        case let .privateKey(key): CopyTypeViewModel(content: privateKeyCopy(key: key))
        }
    }

    var docsUrl: URL {
        AppUrl.docs(.howToSecureSecretPhrase)
    }
}
