// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct PaymentLinkPayableMock: PaymentLinkPayable {
    public init() {}

    public func pay(link _: PaymentLink, wallet _: Wallet) async {}
}

public extension PaymentLinkPayable where Self == PaymentLinkPayableMock {
    static func mock() -> PaymentLinkPayableMock {
        PaymentLinkPayableMock()
    }
}
