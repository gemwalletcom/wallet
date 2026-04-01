// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

extension TransactionExtended: Identifiable {
    public var id: String {
        transaction.id.identifier
    }
}
