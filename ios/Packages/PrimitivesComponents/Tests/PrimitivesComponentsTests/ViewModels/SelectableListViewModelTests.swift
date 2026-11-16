// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemWalletRow
import struct Gemstone.GemWalletSection
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct SelectableListViewModelTests {
    private let networksModel = SelectableListViewModel<Chain>(
        networks: .data(.plain([.bitcoin, .ethereum])),
        selectedItems: [],
        selectionType: .checkmark,
    )

    @Test
    func pinnedWalletsGetTheirOwnSection() {
        let pinned = GemWalletRow.mock(id: .mock(address: "a"), name: "a", subtitle: .address(value: "a"), placeholder: .multicoin, isPinned: true)
        let model = SelectableListViewModel(
            sections: [GemWalletSection(kind: .pinned, rows: [pinned]), GemWalletSection(kind: .wallets, rows: [.mock(id: .mock(address: "b")), .mock(id: .mock(address: "c"))])],
            selectedRow: pinned,
        )

        guard case let .data(.section(sections)) = model.state else {
            Issue.record("expected sections, got \(model.state)")
            return
        }
        #expect(sections.count == 2)
        #expect(sections[0].values.map(\.id) == [WalletId.mock(address: "a")])
        #expect(sections[1].values.map(\.id) == [WalletId.mock(address: "b"), .mock(address: "c")])
        #expect(model.selectedItems == [pinned])
    }

    @Test
    func withNoPinnedWalletThereIsOneSection() {
        let first = GemWalletRow.mock(id: .mock(address: "a"), name: "a", subtitle: .address(value: "a"), placeholder: .multicoin)
        let model = SelectableListViewModel(sections: [GemWalletSection(kind: .wallets, rows: [first, .mock(id: .mock(address: "b"))])], selectedRow: first)

        guard case let .data(.section(sections)) = model.state else {
            Issue.record("expected sections, got \(model.state)")
            return
        }
        #expect(sections.count == 1)
        #expect(sections[0].title == nil)
    }

    @Test
    func searchable() {
        #expect(networksModel.search != nil)
        #expect(networksModel.items == [.bitcoin, .ethereum])
    }

    @Test
    func filtersByQuery() throws {
        let search = try #require(networksModel.search)

        #expect(search.filter([.bitcoin, .ethereum], "bitc") == [.bitcoin])
        #expect(search.filter([.bitcoin], "ethereum").isEmpty)
    }
}
