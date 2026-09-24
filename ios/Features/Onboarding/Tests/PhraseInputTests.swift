// Copyright (c). Gem Wallet. All rights reserved.

@testable import Onboarding
import Testing

struct PhraseInputTests {
    @Test
    func theWordAtTheCursorIsTheOneBeingEdited() {
        let input = PhraseInput(text: "abandon woo zoo", cursor: 11)

        #expect(input.word == "woo")
        #expect(input.wordCursor == 3)
        #expect(PhraseInput(text: "abandon ", cursor: 8).word.isEmpty)
        #expect(PhraseInput(text: "woo", cursor: nil).wordCursor == 3, "no cursor reads as the end")
        #expect(PhraseInput(text: "woo", cursor: 99).cursor == 3, "a stale cursor is clamped")
    }

    @Test
    func completingReplacesTheWordAndMovesPastTheSeparator() {
        #expect(PhraseInput(text: "abandon woo", cursor: nil).completing(with: "wood") == PhraseInput(text: "abandon wood ", cursor: 13))
        #expect(PhraseInput(text: "woo zoo", cursor: 3).completing(with: "wood") == PhraseInput(text: "wood zoo", cursor: 5))
        #expect(PhraseInput(text: "abandon wo zoo", cursor: 9).completing(with: "wool") == PhraseInput(text: "abandon wool zoo", cursor: 13))
        #expect(PhraseInput(text: "wo", cursor: 1).completing(with: "wood") == PhraseInput(text: "wood ", cursor: 5))
    }

    @Test
    func theCursorCountsUtf16Units() {
        let input = PhraseInput(text: "🙂 woo", cursor: 6)

        #expect(input.word == "woo")
        #expect(input.completing(with: "wood") == PhraseInput(text: "🙂 wood ", cursor: 8))
    }
}
