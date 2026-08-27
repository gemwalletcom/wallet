// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemPerpetualSocketUpdate
import Foundation
import Primitives

public protocol HyperliquidPerpetualServiceable: PerpetualServiceable {
    func accountMode(walletId: WalletId, address: String) async -> PerpetualAccountMode
    func applySocketMessage(walletId: WalletId, mode: PerpetualAccountMode, data: Data) async throws -> GemPerpetualSocketUpdate
}
