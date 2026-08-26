// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public protocol HyperliquidPerpetualServiceable: PerpetualServiceable {
    func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode
    func getHypercorePositions(walletId: WalletId) throws -> [Primitives.PerpetualPosition]
    func updateBalance(walletId: WalletId, balance: Primitives.PerpetualBalance) throws
    func diffPositions(deleteIds: [String], positions: [Primitives.PerpetualPosition], walletId: WalletId) throws
    func updateMarket(_ market: Primitives.PerpetualMarketData) throws
    func updatePrices(_ prices: [String: Double]) throws
}
