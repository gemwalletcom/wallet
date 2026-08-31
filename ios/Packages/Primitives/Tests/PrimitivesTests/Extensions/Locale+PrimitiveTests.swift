// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct Locale_PrimitivesTests {
    @Test
    func testAppstoreLanguageIdentifier() {
        #expect(Locale.US.appstoreLanguageIdentifier() == "en-US")
        #expect(Locale.UK.appstoreLanguageIdentifier() == "en-GB")
        #expect(Locale.FR.appstoreLanguageIdentifier() == "fr-CA")
        #expect(Locale.IT.appstoreLanguageIdentifier() == "it")
        #expect(Locale.ZH_Simplifier.appstoreLanguageIdentifier() == "zh-Hans")
        #expect(Locale.ZH_Traditional.appstoreLanguageIdentifier() == "zh-Hant")
        #expect(Locale.AR_SA.appstoreLanguageIdentifier() == "ar-SA")
    }
}
