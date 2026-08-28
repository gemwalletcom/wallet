// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.walletConnectNamespace
import func Gemstone.walletConnectReference
import Primitives
import struct WalletConnectUtils.Blockchain

extension Primitives.Chain {
    /// CAIP-2 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-2.md
    var namespace: String? {
        walletConnectNamespace(chain: rawValue)
    }

    /// CAIP-20 https://github.com/ChainAgnostic/CAIPs/blob/main/CAIPs/caip-20.md
    var reference: String? {
        walletConnectReference(chain: rawValue)
    }

    var blockchain: Blockchain? {
        if let namespace, let reference {
            return Blockchain(namespace: namespace, reference: reference)
        }
        return .none
    }
}
