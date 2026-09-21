// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Foundation
import struct Gemstone.GemSimulationPayloadRow
import enum Gemstone.GemSimulationPayloadTitle
import Localization
import PrimitivesComponents
import Testing

struct SimulationPayloadFieldViewModelTests {
    private let address = "0x2Df1c51E09aECF9cacB7bc98cB1742757f163dF7"

    @Test
    func addressRowShowsItsDisplayAndOffersCopyAndExplorer() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .spender, value: .address(display: "Hyperliquid (0x2Df1...3dF7)", address: address)),
            explorerItem: .url(title: "Etherscan", onOpen: {}),
        )

        #expect(viewModel.title == Localized.Transfer.to)
        #expect(viewModel.subtitle == "Hyperliquid (0x2Df1...3dF7)")
        #expect(viewModel.contextMenuItems.count == 2)

        guard case let .copy(_, value, _, _) = viewModel.contextMenuItems[0] else {
            Issue.record("Expected copy context menu item")
            return
        }

        #expect(value == address)
    }

    @Test
    func timestampRowFormatsTheRelativeDate() {
        let formatter = RelativeDateFormatter()
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .expiration, value: .timestamp(unixMs: 1_662_714_817_000)),
            relativeDateFormatter: formatter,
        )

        #expect(viewModel.title == Localized.Common.expiration)
        #expect(viewModel.subtitle == formatter.string(from: Date(timeIntervalSince1970: 1_662_714_817)))
        #expect(viewModel.contextMenuItems.isEmpty)
    }

    @Test
    func textRowShowsItsTextUnderTheCustomLabel() {
        let viewModel = SimulationPayloadFieldViewModel(
            row: GemSimulationPayloadRow(title: .custom(label: "issuedAt"), value: .text(text: "Set Approval For All")),
        )

        #expect(viewModel.title == "issuedAt")
        #expect(viewModel.subtitle == "Set Approval For All")
        #expect(viewModel.contextMenuItems.isEmpty)
    }

    @Test
    func titlesFollowTheKind() {
        let titles = [GemSimulationPayloadTitle.contract, .method, .token, .value].map {
            SimulationPayloadFieldViewModel(row: GemSimulationPayloadRow(title: $0, value: .text(text: ""))).title
        }

        #expect(titles == [Localized.Asset.contract, Localized.Common.method, Localized.Common.token, Localized.Perpetual.value])
    }
}
