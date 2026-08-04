// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public extension TransactionAppMetadata {
    static func mock(
        name: String = "Test Dapp",
        description: String? = .none,
        url: String? = "https://example.com",
        icon: String? = "https://example.com/icon.png",
    ) -> Self {
        .init(name: name, description: description, url: url, icon: icon)
    }
}
