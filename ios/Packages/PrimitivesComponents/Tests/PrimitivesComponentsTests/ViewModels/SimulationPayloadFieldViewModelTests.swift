// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import func Gemstone.addressCopy
import struct Gemstone.GemSimulationPayloadRow
import enum Gemstone.GemSimulationPayloadTitle
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct SimulationPayloadFieldViewModelTests {
    private let address = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7"
    private let link = BlockExplorerLink(name: "Etherscan", link: "https://etherscan.io/address/0x1")
    private let method = GemSimulationPayloadRow(title: .method, value: .text(text: "approve"))

    private var contract: GemSimulationPayloadRow {
        GemSimulationPayloadRow(title: .contract, value: .address(display: "0x1", copy: addressCopy(chain: Chain.ethereum.rawValue, address: "0x1"), explorer: link.toGem()))
    }

    @Test
    func addressRowShowsItsDisplay() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(
                title: .spender,
                value: .address(display: "Hyperliquid (0x2Df1...3dF7)", copy: addressCopy(chain: Chain.ethereum.rawValue, address: address), explorer: BlockExplorerLink.mock().toGem()),
            ),
        )

        #expect(viewModel.title == Localized.Transfer.to)
        #expect(viewModel.subtitle == "Hyperliquid (0x2Df1...3dF7)")
    }

    @Test
    func timestampRowReadsThroughTheRowDateRenderer() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .expiration, value: .timestamp(unixMs: 1_662_714_817_000)),
        )

        #expect(viewModel.title == Localized.Common.expiration)
        #expect(viewModel.subtitle == TransactionDateFormatter(date: Date(timeIntervalSince1970: 1_662_714_817)).row)
    }

    @Test
    func aMissingTimestampReadsAsNothingRatherThan1970() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .expiration, value: .timestamp(unixMs: 0)),
        )

        #expect(viewModel.subtitle.isEmpty)
    }

    @Test
    func textRowShowsItsTextUnderTheCustomLabel() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .custom(label: "issuedAt"), value: .text(text: "Set Approval For All")),
        )

        #expect(viewModel.title == "issuedAt")
        #expect(viewModel.subtitle == "Set Approval For All")
    }

    @Test
    func titlesFollowTheKind() {
        let titles = [GemSimulationPayloadTitle.contract, .method, .token, .value].map {
            SimulationPayloadFieldViewModel(row: GemSimulationPayloadRow(title: $0, value: .text(text: ""))).title
        }

        #expect(titles == [Localized.Asset.contract, Localized.Common.method, Localized.Common.token, Localized.Perpetual.value])
    }

    @Test
    func addressFieldCopiesAndOpensTheRowsExplorerLink() {
        let kind = SimulationPayloadFieldViewModel.models(for: [contract])[0].kind

        #expect(kind == .address(ExplorerContextData(copyValue: .address(value: "0x1", chain: .ethereum), explorerLink: link)))
    }

    @Test
    func textFieldIsPlain() {
        let model = SimulationPayloadFieldViewModel.models(for: [method])[0]

        #expect(model.kind == .plain)
        #expect(model.onSelect == nil)
    }

    @Test
    @MainActor
    func addressFieldSelectsTheAddress() {
        var selected: String?

        let model = SimulationPayloadFieldViewModel.models(for: [contract], onSelectAddress: { selected = $0 })[0]

        model.onSelect?()
        #expect(selected == "0x1")
    }

    @Test
    func textFieldDoesNotSelectAnAddress() {
        let model = SimulationPayloadFieldViewModel.models(for: [method], onSelectAddress: { _ in })[0]

        #expect(model.onSelect == nil)
    }
}
