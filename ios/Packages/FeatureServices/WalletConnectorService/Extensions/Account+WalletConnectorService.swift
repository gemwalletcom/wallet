// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Account
import class Gemstone.WalletConnect
import Primitives
import ReownWalletKit

extension Primitives.Account {
    var blockchain: WalletConnectUtils.Account? {
        if let blockchain = chain.blockchain {
            return WalletConnectUtils.Account(blockchain: blockchain, address: address)
        }
        return .none
    }

    public func mapToGem() -> Gemstone.Account {
        Gemstone.Account(
            chain: chain.rawValue,
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: extendedPublicKey,
        )
    }
}
