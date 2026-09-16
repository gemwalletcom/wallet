// Copyright (c). Gem Wallet. All rights reserved.

import PrimitivesComponents
import Testing

struct SecretPhraseRowTests {
    @Test
    func testRows() {
        #expect(SecretPhraseRow.rows(for: ["a", "b", "c", "d"]) == [
            .pair(WordIndex(index: 0, word: "a"), WordIndex(index: 2, word: "c")),
            .pair(WordIndex(index: 1, word: "b"), WordIndex(index: 3, word: "d")),
        ])
        #expect(SecretPhraseRow.rows(for: ["a", "b", "c", "d", "e"]) == [
            .pair(WordIndex(index: 0, word: "a"), WordIndex(index: 2, word: "c")),
            .pair(WordIndex(index: 1, word: "b"), WordIndex(index: 3, word: "d")),
            .single(WordIndex(index: 4, word: "e")),
        ])
        #expect(SecretPhraseRow.rows(for: []) == [])
    }
}
