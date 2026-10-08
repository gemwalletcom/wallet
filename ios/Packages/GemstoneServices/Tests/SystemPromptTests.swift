// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Primitives
import Testing

struct SystemPromptTests {
    @Test
    func aFailedPromptStillEnds() {
        let systemPrompt = SystemPrompt()

        #expect(throws: AnyError("cancelled")) { try systemPrompt.presenting { throw AnyError("cancelled") } }

        #expect(!systemPrompt.hasPresented(since: .now))
    }

    @Test
    func overlappingPromptsStayPresentedUntilTheLastEnds() {
        let systemPrompt = SystemPrompt()

        let afterInnerEnded = systemPrompt.presenting {
            systemPrompt.presenting {}
            return systemPrompt.hasPresented(since: .now)
        }

        #expect(afterInnerEnded)
        #expect(!systemPrompt.hasPresented(since: .now))
    }

    @Test
    func anEndedPromptCountsOnlyForEarlierInstants() {
        let systemPrompt = SystemPrompt()
        let before = ContinuousClock.now
        #expect(!systemPrompt.hasPresented(since: before))

        systemPrompt.presenting {}

        #expect(systemPrompt.hasPresented(since: before))
        #expect(!systemPrompt.hasPresented(since: .now))
    }
}
