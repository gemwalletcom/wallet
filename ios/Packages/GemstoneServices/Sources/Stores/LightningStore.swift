// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemLightningStore
import struct Gemstone.LightningPendingSend
import GemstonePrimitives
import Primitives
import Store

public final class GemstoneLightningStore: GemLightningStore, @unchecked Sendable {
    private let store: LightningStore

    public init(store: LightningStore) {
        self.store = store
    }

    public func getPendingSends(walletId: WalletId) async throws -> [Gemstone.LightningPendingSend] {
        try store.getPendingSends(walletId: walletId).map {
            Gemstone.LightningPendingSend(transferId: $0.transferId, invoice: $0.invoice, amountSats: $0.amountSats, expiresAt: $0.expiresAt)
        }
    }

    public func addPendingSend(walletId: WalletId, send: Gemstone.LightningPendingSend) async throws {
        try store.addPendingSend(
            walletId: walletId,
            send: Store.LightningPendingSend(transferId: send.transferId, invoice: send.invoice, amountSats: send.amountSats, expiresAt: send.expiresAt),
        )
    }

    public func deletePendingSend(walletId: WalletId, transferId: String) async throws {
        try store.deletePendingSend(walletId: walletId, transferId: transferId)
    }
}
