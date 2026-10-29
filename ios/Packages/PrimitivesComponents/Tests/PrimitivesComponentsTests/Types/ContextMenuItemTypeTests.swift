// Copyright (c). Gem Wallet. All rights reserved.

@testable import PrimitivesComponents
import SwiftUI
import Testing

struct ContextMenuItemTypeTests {
    @Test
    func aMenuSplitsWhereTheGroupChangesAndKeepsItsOrder() {
        let wallet: [ContextMenuItemType] = [
            .custom(title: "Settings", action: nil),
            .pin(isPinned: false, onPin: nil),
            .delete(nil),
        ]
        let address: [ContextMenuItemType] = [
            .copy(value: "0x1"),
            .url(title: "Etherscan", onOpen: nil),
            .custom(title: "Create New Contact", action: nil),
            .custom(title: "Report", role: .destructive, action: nil),
        ]

        #expect(wallet.runs(by: \.group).map { $0.map(\.group) } == [[.actions, .actions], [.destructive]])
        #expect(address.runs(by: \.group).map { $0.map(\.group) } == [[.view, .view], [.actions], [.destructive]])
        #expect([ContextMenuItemType.copy(value: "0x1")].runs(by: \.group).count == 1)
    }
}
