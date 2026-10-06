// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemChainServiceProtocol
import Primitives
import ReownWalletKit

extension Primitives.Account {
    func blockchains(chainService: any GemChainServiceProtocol) -> [WalletConnectUtils.Account] {
        chain.blockchains(chainService: chainService).compactMap {
            WalletConnectUtils.Account(blockchain: $0, address: address)
        }
    }
}
