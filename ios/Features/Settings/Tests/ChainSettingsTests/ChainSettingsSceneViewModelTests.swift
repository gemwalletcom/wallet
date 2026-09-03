// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Primitives
@testable import Settings
import Testing

@MainActor
struct ChainSettingsSceneViewModelTests {
    @Test
    func loadsSelectedNodeByUrl() async throws {
        let model = ChainSettingsSceneViewModel.mock()

        await model.load()

        let first = try #require(model.nodesModels.first)
        #expect(model.nodesModels.allSatisfy { $0.chainNode.host == first.chainNode.host })
        #expect(model.selectedNode.node.url == first.chainNode.node.url)
        #expect(model.nodesModels.count(where: { $0.chainNode == model.selectedNode }) == 1)
    }

    @Test
    func selectsSameHostNode() async throws {
        let model = ChainSettingsSceneViewModel.mock()
        await model.load()
        let last = try #require(model.nodesModels.last)

        model.onSelectNode(last.chainNode)

        #expect(model.selectedNode.node.url == last.chainNode.node.url)
        #expect(model.nodesModels.count(where: { $0.chainNode == model.selectedNode }) == 1)
    }
}

// MARK: - Mock

extension ChainSettingsSceneViewModel {
    static func mock() -> ChainSettingsSceneViewModel {
        ChainSettingsSceneViewModel(
            chain: .ethereum,
            service: GemChainSettingsServiceMock(nodeUrls: ["https://rpc.example.com/one", "https://rpc.example.com/two"]),
        )
    }
}
