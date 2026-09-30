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
        service.nodesByCall = [[.mock(url: "a", host: "a"), .mock(url: "b", host: "b")]]
        service.statusByUrl = ["a": .result(latestBlockNumber: 10, latency: Latency(latencyType: .fast, value: 5)), "b": .error]
        let model = ChainSettingsSceneViewModel(chain: .ethereum, service: service)

        await model.load()

        #expect(model.nodeRows.count == 2)
        #expect(service.statusCalls.sorted() == ["a", "b"])
        #expect(model.nodeRows[0].subtitle == GemNodeSubtitle.latestBlock(value: .mock(value: 10, unit: .plain, display: .number(precision: .fraction(min: 0, max: 0)), notation: .plain, tone: .plain, rounding: .toNearest)))
    }

    @Test
    func aDeletedNodeLeavesNoRowBehind() async throws {
        let service = GemChainSettingsServiceMock()
        service.nodesByCall = [[.mock(url: "a", host: "a"), .mock(url: "b", host: "b")], [.mock(url: "a", host: "a")]]
        service.statusByUrl = ["a": .error, "b": .error]
        let model = ChainSettingsSceneViewModel(chain: .ethereum, service: service)
        await model.load()

        try model.onSelectNodeForDeletion(#require(model.nodeRows.first { $0.node.url == "b" }))
        await model.onDeleteNode()

        #expect(service.deletedNodes == ["b"])
        #expect(model.nodeRows.map(\.node.url) == ["a"])
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

private extension ChainSettingsSceneViewModel {
    var nodeRows: [GemNodeRow] {
        sections.flatMap { section -> [GemNodeRow] in
            if case let .nodes(rows) = section {
                rows
            } else {
                []
            }
        }
    }
}
