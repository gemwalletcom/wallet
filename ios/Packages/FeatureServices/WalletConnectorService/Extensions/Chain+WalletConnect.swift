// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemChainService
import Primitives
import struct WalletConnectUtils.Blockchain

extension Primitives.Chain {
    /// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
    func namespace(chainService: GemChainService) -> String? {
        chainService.caip2Namespace(chain: rawValue)
    }

    /// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
    func reference(chainService: GemChainService) -> String? {
        chainService.caip2Reference(chain: rawValue)
    }

    func blockchain(chainService: GemChainService) -> Blockchain? {
        guard let namespace = namespace(chainService: chainService), let reference = reference(chainService: chainService) else {
            return .none
        }
        return Blockchain(namespace: namespace, reference: reference)
    }
}
