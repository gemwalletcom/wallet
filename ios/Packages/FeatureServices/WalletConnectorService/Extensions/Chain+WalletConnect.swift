// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemChainServiceProtocol
import Primitives
import struct WalletConnectUtils.Blockchain

extension Primitives.Chain {
    func blockchains(chainService: any GemChainServiceProtocol) -> [Blockchain] {
        guard let namespace = chainService.caip2Namespace(chain: rawValue) else {
            return []
        }
        return chainService.caip2References(chain: rawValue).compactMap {
            Blockchain(namespace: namespace, reference: $0)
        }
    }
}
