// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension WalletConnectSessionProposal: Identifiable {
    public var id: String {
        metadata.url
    }
}
