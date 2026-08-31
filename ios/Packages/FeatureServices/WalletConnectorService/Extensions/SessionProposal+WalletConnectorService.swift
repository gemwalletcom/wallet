// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectSign

extension [String: ProposalNamespace] {
    var chainIds: [String] {
        values.flatMap { $0.chains ?? [] }.map(\.absoluteString)
    }
}
