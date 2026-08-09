// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct Locale_PrimitivesTests {
    @Test
    func deviceLocale() {
        #expect(Locale.US.deviceLocale() == .en)
        #expect(Locale.UK.deviceLocale() == .en)
        #expect(Locale.EN_CH.deviceLocale() == .en)
        #expect(Locale(identifier: "in_ID").deviceLocale() == .id)
        #expect(Locale(identifier: "iw_IL").deviceLocale() == .he)
        #expect(Locale.PT_BR.deviceLocale() == .ptBR)
        #expect(Locale.PT_PT.deviceLocale() == .ptBR)
        #expect(Locale(identifier: "tl_PH").deviceLocale() == .fil)
        #expect(Locale.FR.deviceLocale() == .fr)
        #expect(Locale.ZH_Simplifier.deviceLocale() == .zhHans)
        #expect(Locale.ZH_Singapore.deviceLocale() == .zhHans)
        #expect(Locale.ZH_Traditional.deviceLocale() == .zhHant)
        #expect(Locale(identifier: "zh_TW").deviceLocale() == .zhHant)
        #expect(Locale(identifier: "zh_HK").deviceLocale() == .zhHant)
        #expect(Locale(identifier: "zh_MO").deviceLocale() == .zhHant)
        #expect(Locale(identifier: "af_ZA").deviceLocale() == .en)
    }

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
