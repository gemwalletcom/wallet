// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension Gemstone.UrlAction {
    func map() throws -> Primitives.URLAction {
        switch self {
        case let .deeplink(deeplink): try .deeplink(deeplink.map())
        case let .payment(payment): try .payment(Primitives.Payment(payment))
        case let .walletConnect(link): .walletConnect(link.map())
        }
    }
}
