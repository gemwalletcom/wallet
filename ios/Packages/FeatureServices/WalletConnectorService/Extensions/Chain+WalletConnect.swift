// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemWalletConnectRulesService
import Primitives
import struct WalletConnectUtils.Blockchain

private let walletConnectRules = GemWalletConnectRulesService()

extension Primitives.Chain {
    /// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
    var namespace: String? {
        walletConnectRules.namespace(chain: rawValue)
    }

    /// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
    var reference: String? {
        walletConnectRules.reference(chain: rawValue)
    }

    var blockchain: Blockchain? {
        if let namespace, let reference {
            return Blockchain(namespace: namespace, reference: reference)
        }
        return .none
    }
}
