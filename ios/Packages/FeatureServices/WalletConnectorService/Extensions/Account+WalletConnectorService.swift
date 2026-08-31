// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.Account
import class Gemstone.GemChainService
import Primitives
import ReownWalletKit

extension Primitives.Account {
    func blockchain(chainService: GemChainService) -> WalletConnectUtils.Account? {
        guard let blockchain = chain.blockchain(chainService: chainService) else {
            return .none
        }
        return WalletConnectUtils.Account(blockchain: blockchain, address: address)
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
