// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
@testable import Support
import SupportChatService
import Testing

@MainActor
struct SupportChatSceneViewModelTests {
    @Test
    func agentCountCountsOnlyAgents() {
        #expect(model([user("1"), user("2"), agent("3"), user("4")]).agentCount == 1)
        #expect(model([agent("1"), agent("2"), agent("3")]).agentCount == 3)
        #expect(model([]).agentCount == 0)
        #expect(model([user("1"), user("2")]).agentCount == 0)
    }

    @Test
    func unreadStartsAtAgentCountAndResetsOnlyAtBottom() {
        let viewModel = model([agent("1"), agent("2"), agent("3"), agent("4"), agent("5")])
        #expect(viewModel.unreadAgentCount == 5)

        viewModel.setAtBottom(false)
        #expect(viewModel.unreadAgentCount == 5)

        viewModel.setAtBottom(true)
        #expect(viewModel.unreadAgentCount == 0)
    }

    @Test
    func unreadNeverNegativeAndTracksNewAgents() {
        let viewModel = model([agent("1"), agent("2")])
        viewModel.setAtBottom(true)
        #expect(viewModel.unreadAgentCount == 0)

        viewModel.query.value = [agent("1"), agent("2"), agent("3"), agent("4"), agent("5")]
        #expect(viewModel.unreadAgentCount == 3)

        viewModel.query.value = [agent("1")]
        #expect(viewModel.unreadAgentCount == 0)
    }
}

// MARK: - Helpers

private extension SupportChatSceneViewModelTests {
    func model(_ messages: [SupportMessage]) -> SupportChatSceneViewModel {
        let viewModel = SupportChatSceneViewModel(service: SupportChatService(store: SupportChatStore(db: .mock())))
        viewModel.query.value = messages
        return viewModel
    }

    func user(_ id: String) -> SupportMessage {
        .mock(id: id, sender: .user)
    }

    func agent(_ id: String) -> SupportMessage {
        .mock(id: id, sender: .agent(.mock(name: "Gemma")))
    }
}
