// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Localization
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import PrimitivesTestKit
import Testing

struct SimulationPayloadModelTests {
    @Test
    func emptyFieldsHaveNoDetails() {
        #expect(!SimulationPayloadModel.mock().hasFields)
        #expect(!SimulationPayloadModel.mock().hasDetails)
    }

    @Test
    func primaryOnlyFieldsHaveNoDetails() {
        let contract = SimulationPayloadField.standard(kind: .contract, value: "0x1", fieldType: .address, display: .primary)
        let model = SimulationPayloadModel.mock(primaryFields: [contract])

        #expect(model.hasFields)
        #expect(!model.hasDetails)
    }

    @Test
    func secondaryOnlyFieldsHaveDetails() {
        let method = SimulationPayloadField.standard(kind: .method, value: "approve", fieldType: .text, display: .secondary)
        let model = SimulationPayloadModel.mock(secondaryFields: [method])

        #expect(model.hasFields)
        #expect(model.hasDetails)
    }

    @Test
    func addressRequestsCoverBothSectionsAndSkipNonAddressFields() {
        let contract = SimulationPayloadField.standard(kind: .contract, value: "0x1", fieldType: .address, display: .primary)
        let method = SimulationPayloadField.standard(kind: .method, value: "approve", fieldType: .text, display: .primary)
        let spender = SimulationPayloadField.standard(kind: .spender, value: "0x2", fieldType: .address, display: .secondary)
        let model = SimulationPayloadModel.mock(chain: .arbitrum, primaryFields: [contract, method], secondaryFields: [spender])

        #expect(model.addressRequests == [
            ChainAddress(chain: .arbitrum, address: contract.value),
            ChainAddress(chain: .arbitrum, address: spender.value),
        ])
    }

    @Test
    func fieldViewModelResolvesAddressName() {
        let contract = SimulationPayloadField.standard(
            kind: .contract,
            value: "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7",
            fieldType: .address,
            display: .primary,
        )
        let spender = SimulationPayloadField.standard(kind: .spender, value: "0x1", fieldType: .address, display: .primary)
        let model = SimulationPayloadModel.mock(
            primaryFields: [contract, spender],
            addressNames: [ChainAddress(chain: .ethereum, address: contract.value): .mock(address: contract.value, name: "Hyperliquid")],
        )

        #expect(model.fieldViewModel(for: contract).addressName?.name == "Hyperliquid")
        #expect(model.fieldViewModel(for: spender).addressName == nil)
    }

    @Test
    func addressFieldContextMenuOpensExplorerLink() {
        let contract = SimulationPayloadField.standard(kind: .contract, value: "0x1", fieldType: .address, display: .primary)
        let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/\(contract.value)")
        var openedURL: URL?

        let items = SimulationPayloadModel.mock(primaryFields: [contract]).contextMenuItems(
            for: contract,
            explorerLink: { _ in link },
            onOpenURL: { openedURL = $0 },
        )

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
        let method = SimulationPayloadField.standard(kind: .method, value: "approve", fieldType: .text, display: .secondary)

        let items = SimulationPayloadModel.mock(secondaryFields: [method]).contextMenuItems(
            for: method,
            explorerLink: { BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/\($0)") },
            onOpenURL: { _ in },
        )

        #expect(items.isEmpty)
    }
}
