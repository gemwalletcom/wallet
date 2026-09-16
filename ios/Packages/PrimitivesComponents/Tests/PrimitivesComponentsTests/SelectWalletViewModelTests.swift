// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import Primitives
@testable import PrimitivesComponents
import Testing

struct SelectWalletViewModelTests {
    private func row(_ id: String, isPinned: Bool = false) -> GemWalletRow {
        GemWalletRow(
            id: id,
            name: id,
            subtitle: .address(value: id),
            placeholder: .multicoin,
            showsWatchBadge: false,
            isPinned: isPinned,
            hasAvatar: false,
            imageUrl: nil,
        )
    }

    @Test
    func pinnedWalletsGetTheirOwnSection() throws {
        let pinned = row("a", isPinned: true)
        let model = SelectWalletViewModel(rows: [pinned, row("b"), row("c")], selectedRow: pinned)

        guard case let .data(.section(sections)) = model.state else {
            Issue.record("expected sections, got \(model.state)")
            return
        }
        #expect(sections.count == 2)
        #expect(sections[0].values.map(\.id) == ["a"])
        #expect(sections[1].values.map(\.id) == ["b", "c"])
        #expect(model.selectedItems == [pinned])
    }

    @Test
    func withNoPinnedWalletThereIsOneSection() {
        let first = row("a")
        let model = SelectWalletViewModel(rows: [first, row("b")], selectedRow: first)

        guard case let .data(.section(sections)) = model.state else {
            Issue.record("expected sections, got \(model.state)")
            return
        }
        #expect(sections.count == 1)
        #expect(sections[0].title == nil)
    }
}
