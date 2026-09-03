// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension Primitives.Account {
    func map() -> Gemstone.Account {
        Gemstone.Account(
            chain: chain.rawValue,
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: extendedPublicKey,
        )
    }
}

public extension GemTransferData {
    func confirmInput(from account: Primitives.Account) -> GemConfirmInput {
        GemConfirmInput(from: account.map(), transfer: self)
    }
}

public extension FeeSelection {
    func map() -> GemConfirmFeeSelection {
        switch self {
        case let .preset(priority): .priority(priority: priority.map())
        case let .custom(gasPrice): .custom(gasPrice: gasPrice)
        }
    }
}
