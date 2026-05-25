// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.WalletConnectLink {
    func map() -> Primitives.WalletConnectAction {
        switch self {
        case let .connect(uri): .connect(uri: uri)
        case .request: .request
        case let .session(topic): .session(topic)
        }
    }
}
