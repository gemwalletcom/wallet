// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol PaymentLinkPayable: Sendable {
    func pay(link: PaymentLink, wallet: Wallet) async
}
