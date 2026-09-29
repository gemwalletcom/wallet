// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemChainService
import Primitives
import struct WalletConnectUtils.Blockchain

extension Primitives.Chain {
    func namespace(chainService: GemChainService) -> String? {
        chainService.caip2Namespace(chain: rawValue)
    }

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
