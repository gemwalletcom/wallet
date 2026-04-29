// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Onboarding
import Testing

struct WordSuggesterTests {
    let suggester = WordSuggester()

    @Test
    func suggestPartial() {
        #expect(suggester.wordSuggestionCalculate(value: "ab").isNotEmpty)
    }

    @Test
    func emptyWhenEndsWithSpace() {
        #expect(suggester.wordSuggestionCalculate(value: "ab ").isEmpty)
    }

    @Test
    func exactMatchReturnsEmpty() {
        #expect(suggester.wordSuggestionCalculate(value: "abandon").isEmpty)
    }

    @Test
    func replaceLastWord() {
        #expect(suggester.selectWordCalculate(input: "abando", word: "abandon") == "abandon ")
    }
}
