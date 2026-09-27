// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import SwiftUI

public extension NavigationPath {
    mutating func append(transfer route: TransferRoute) {
        switch route {
        case let .amount(input): append(input)
        case let .confirm(data): append(ConfirmTransferInput(data: data))
        }
    }
}
