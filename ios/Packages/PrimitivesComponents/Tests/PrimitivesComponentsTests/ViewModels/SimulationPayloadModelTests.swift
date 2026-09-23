// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemSimulationPayloadRow
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import PrimitivesComponentsTestKit
import Testing

struct SimulationPayloadModelTests {
    private let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/0x1")
    private let contract: GemSimulationPayloadRow
    private let method = GemSimulationPayloadRow(title: .method, value: .text(text: "approve"))

    init() {
        let copy = addressCopy(chain: Chain.ethereum.rawValue, address: "0x1")
        contract = GemSimulationPayloadRow(title: .contract, value: .address(display: "0x1", copy: copy, explorer: link.toGem()))
    }

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
    func addressFieldCopiesAndOpensTheRowsExplorerLink() {
        let kind = SimulationPayloadModel.mock(primaryFields: [contract]).fieldModels(for: [contract])[0].kind

        #expect(kind == .address(ExplorerContextData(copyValue: .address(value: "0x1", chain: .ethereum), explorerLink: link)))
    }

    @Test
    func textFieldIsPlain() {
        let model = SimulationPayloadModel.mock(secondaryFields: [method]).fieldModels(for: [method])[0]

        #expect(model.kind == .plain)
        #expect(model.onSelect == nil)
    }

    @Test
    @MainActor
    func addressFieldSelectsTheAddress() {
        var selected: String?

        let model = SimulationPayloadModel.mock(primaryFields: [contract]).fieldModels(
            for: [contract],
            onSelectAddress: { selected = $0 },
        )[0]

        model.onSelect?()
        #expect(selected == "0x1")
    }

    @Test
    func textFieldDoesNotSelectAnAddress() {
        let model = SimulationPayloadModel.mock(secondaryFields: [method]).fieldModels(
            for: [method],
            onSelectAddress: { _ in },
        )[0]

        #expect(model.onSelect == nil)
    }
}
