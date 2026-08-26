// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public protocol HyperliquidPerpetualServiceable: PerpetualServiceable {
    func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode
    func getHypercorePositions(walletId: WalletId) async throws -> [Primitives.PerpetualPosition]
    func updateBalance(walletId: WalletId, balance: Primitives.PerpetualBalance) async throws
    func updatePositions(walletId: WalletId, positions: [Primitives.PerpetualPosition], deleteIds: [String]) async throws
    func updateMarket(_ market: Primitives.PerpetualMarketData) async throws
    func updatePrices(_ prices: [String: Double]) async throws
}
