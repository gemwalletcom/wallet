// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import GemstonePrimitivesTestKit
@testable import Primitives
@testable import Settings
import Testing

@MainActor
struct ChainSettingsSceneViewModelTests {
    @Test
    func everyNodeGetsItsStatus() async {
        let service = GemChainSettingsServiceMock()
        service.nodesByCall = [[.mock(url: "a"), .mock(url: "b")]]
        service.statusByUrl = ["a": .result(latestBlockNumber: 10, latency: Latency(latencyType: .fast, value: 5)), "b": .error]
        let model = ChainSettingsSceneViewModel(chain: .ethereum, service: service)

        await model.load()

        #expect(model.nodesModels.count == 2)
        #expect(service.statusCalls.sorted() == ["a", "b"])
        #expect(model.nodesModels[0].row.subtitle == GemNodeSubtitle.latestBlock(value: .mock(value: 10, unit: .plain, display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain)))
    }

    @Test
    func aDeletedNodeLeavesNoRowBehind() async {
        let service = GemChainSettingsServiceMock()
        service.nodesByCall = [[.mock(url: "a"), .mock(url: "b")], [.mock(url: "a")]]
        service.statusByUrl = ["a": .error, "b": .error]
        let model = ChainSettingsSceneViewModel(chain: .ethereum, service: service)
        await model.load()

        model.onSelectNodeForDeletion(.mock(url: "b"))
        await model.onDeleteNode()

        #expect(service.deletedNodes == ["b"])
        #expect(model.nodesModels.map(\.node.url) == ["a"])
    }

    @Test
    func choosingAnExplorerReloadsTheRows() {
        let service = GemChainSettingsServiceMock()
        service.explorerRowsValue = [GemExplorerRow(name: "Etherscan", isSelected: true)]
        let model = ChainSettingsSceneViewModel(chain: .ethereum, service: service)

        service.explorerRowsValue = [GemExplorerRow(name: "Blockchair", isSelected: true)]
        model.onSelectExplorer(name: "Blockchair")

        #expect(service.setExplorerNames == ["Blockchair"])
        #expect(model.explorers.map(\.name) == ["Blockchair"])
    }
}
