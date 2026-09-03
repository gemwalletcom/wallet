// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletPreferencesServiceProtocol
import Primitives

public extension GemWalletPreferencesServiceProtocol {
    func getPerpetualAccountMode(walletId: WalletId) throws -> Primitives.PerpetualAccountMode {
        try Primitives.PerpetualAccountMode(getPerpetualAccountMode(walletId: walletId.id))
    }
}
