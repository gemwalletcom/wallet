// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
@testable import Store
import StoreTestKit
import Testing

struct LightningStoreTests {
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
