// Copyright (c). Gem Wallet. All rights reserved.

import Localization
@testable import PrimitivesComponents
import Testing

struct SecretDataViewModelTests {
    @Test
    func anExportedPhraseShowsItsWordsUnderTheSecretPhraseTitle() {
        let model = SecretDataViewModel(secret: .words(words: ["alpha", "beta"]))

        #expect(model.title == Localized.Common.secretPhrase)
        #expect(model.continueAction == nil)
        guard case let .words(rows) = model.type else {
            Issue.record("expected the phrase words")
            return
        }
        #expect(rows == SecretPhraseRow.rows(for: ["alpha", "beta"]))
    }

    @Test
    func aNewPhraseAsksToContinue() {
        let model = SecretDataViewModel(secret: .words(words: ["alpha"]), continueAction: {})

        #expect(model.title == Localized.Wallet.New.title)
        #expect(model.continueAction != nil)
    }

    @Test
    func aPrivateKeyShowsTheKey() {
        let model = SecretDataViewModel(secret: .privateKey(key: "0xkey"))

        #expect(model.title == Localized.Common.privateKey)
        guard case let .privateKey(key) = model.type else {
            Issue.record("expected the private key")
            return
        }
        #expect(key == "0xkey")
    }
}
