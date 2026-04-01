// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import WalletCore

public extension SignerInput {
    var coinType: WalletCore.CoinType {
        asset.chain.coinType
    }
}
