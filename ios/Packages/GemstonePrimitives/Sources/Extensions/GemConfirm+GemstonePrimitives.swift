// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemTransferData {
    func confirmInput(from account: Primitives.Account) -> GemConfirmInput {
        GemConfirmInput(from: account.map(), transfer: self)
    }
}
