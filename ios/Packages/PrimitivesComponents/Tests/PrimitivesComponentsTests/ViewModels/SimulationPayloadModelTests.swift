// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemSimulationPayloadRow
import Localization
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

struct SimulationPayloadModelTests {
    private let contract = GemSimulationPayloadRow(title: .contract, value: .address(display: "0x1", address: "0x1"))
    private let method = GemSimulationPayloadRow(title: .method, value: .text(text: "approve"))

    @Test
    func emptyFieldsHaveNoDetails() {
        #expect(!SimulationPayloadModel.mock().hasFields)
        #expect(!SimulationPayloadModel.mock().hasDetails)
    }

    @Test
    func primaryOnlyFieldsHaveNoDetails() {
        let model = SimulationPayloadModel.mock(primaryFields: [contract])

        #expect(model.hasFields)
        #expect(!model.hasDetails)
    }

    @Test
    func secondaryOnlyFieldsHaveDetails() {
        let model = SimulationPayloadModel.mock(secondaryFields: [method])

        #expect(model.hasFields)
        #expect(model.hasDetails)
    }

    @Test
    func addressFieldContextMenuOpensExplorerLink() {
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/0x1")
        var openedURL: URL?

        let items = SimulationPayloadModel.mock(primaryFields: [contract]).fieldModels(
            for: [contract],
            explorerLink: { _ in link },
            onOpenURL: { openedURL = $0 },
        )[0].contextMenuItems

        #expect(items.count == 2)
        guard case let .url(title, onOpen) = items[1], let onOpen else {
            Issue.record("Expected explorer url context menu item")
            return
        }

        #expect(title == Localized.Transaction.viewOn(link.name))
        onOpen()
        #expect(openedURL == URL(string: link.link))
    }

    @Test
    func textFieldContextMenuOmitsExplorerLink() {
        let items = SimulationPayloadModel.mock(secondaryFields: [method]).fieldModels(
            for: [method],
            explorerLink: { BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/\($0)") },
            onOpenURL: { _ in },
        )[0].contextMenuItems

        #expect(items.isEmpty)
    }
}
