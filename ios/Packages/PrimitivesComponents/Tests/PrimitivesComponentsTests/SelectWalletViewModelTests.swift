// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Gemstone
import GemstonePrimitivesTestKit
import Primitives
@testable import PrimitivesComponents
import Testing

struct SelectWalletViewModelTests {
    @Test
    func pinnedWalletsGetTheirOwnSection() {
        let pinned = GemWalletRow.mock(id: "a", isPinned: true)
        let model = SelectWalletViewModel(rows: [pinned, .mock(id: "b"), .mock(id: "c")], selectedRow: pinned)

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
        let first = GemWalletRow.mock(id: "a")
        let model = SelectWalletViewModel(rows: [first, .mock(id: "b")], selectedRow: first)

        guard case let .data(.section(sections)) = model.state else {
            Issue.record("expected sections, got \(model.state)")
            return
        }
        #expect(sections.count == 1)
        #expect(sections[0].title == nil)
    }
}
