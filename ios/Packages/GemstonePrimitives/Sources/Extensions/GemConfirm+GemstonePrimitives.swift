// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
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

public extension TransferData {
    func confirmInput(from account: Primitives.Account) -> GemConfirmInput {
        GemConfirmInput(from: account.map(), transfer: gem)
    }
}

public extension FeeSelection {
    func map() -> GemConfirmFeeSelection {
        switch self {
        case let .preset(priority): .priority(priority: priority.rawValue)
        case let .custom(gasPrice): .custom(gasPrice: gasPrice.description)
        }
    }
}
