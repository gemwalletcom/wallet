// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.WalletConnect
import Primitives
import ReownWalletKit

extension Primitives.Account {
    var blockchain: WalletConnectUtils.Account? {
        if let blockchain = chain.blockchain {
            return Account(blockchain: blockchain, address: address)
        }
        return .none
    }
}

extension WalletConnectUtils.Account {
    var chain: Primitives.Chain? {
        guard let account = WalletConnect.shared.parseAccount(account: absoluteString) else {
            return .none
        }
        return Primitives.Chain(rawValue: account.chain)
    }
}
