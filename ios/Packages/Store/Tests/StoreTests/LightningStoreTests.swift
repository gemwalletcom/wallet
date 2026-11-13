// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct LightningStoreTests {
    @Test
    func leavesAreKeptPerWalletAndDeletedWithIt() throws {
        let wallet = Wallet.mock()
        let otherWallet = Wallet.mock(id: .mock(address: "0x2"))
        let db = DB.mock(wallets: [wallet, otherWallet])
        let store = LightningStore(db: db)
        let leaf = LightningLeaf(id: "leaf", data: "{}")

        try store.updateLeaves(walletId: wallet.id, leaves: [leaf, LightningLeaf(id: "spent", data: "{}")], deleteIds: [])
        try store.updateLeaves(walletId: otherWallet.id, leaves: [leaf], deleteIds: [])
        try store.updateLeaves(walletId: wallet.id, leaves: [LightningLeaf(id: "leaf", data: "{\"value\":1}")], deleteIds: ["spent"])

        #expect(try store.getLeaves(walletId: wallet.id) == [LightningLeaf(id: "leaf", data: "{\"value\":1}")])
        #expect(try store.getLeaves(walletId: otherWallet.id) == [leaf])

        _ = try WalletStore(db: db).deleteWallet(for: wallet.id)

        #expect(try store.getLeaves(walletId: wallet.id).isEmpty)
        #expect(try store.getLeaves(walletId: otherWallet.id) == [leaf])
    }

    @Test
    func aPendingSendIsRecordedOncePerTransfer() throws {
        let wallet = Wallet.mock()
        let store = LightningStore(db: .mock(wallets: [wallet]))
        let send = LightningPendingSend(transferId: "1", invoice: "lnbc1", amountSats: nil, expiresAt: 1_800_000_000)
        let other = LightningPendingSend(transferId: "2", invoice: "lnbc1", amountSats: 2000, expiresAt: 1_800_000_000)

        try store.addPendingSend(walletId: wallet.id, send: send)
        try store.addPendingSend(walletId: wallet.id, send: other)

        #expect(try store.getPendingSends(walletId: wallet.id) == [send, other])
        #expect(throws: (any Error).self) { try store.addPendingSend(walletId: wallet.id, send: send) }

        try store.deletePendingSend(walletId: wallet.id, transferId: send.transferId)

        #expect(try store.getPendingSends(walletId: wallet.id) == [other])
    }
}
