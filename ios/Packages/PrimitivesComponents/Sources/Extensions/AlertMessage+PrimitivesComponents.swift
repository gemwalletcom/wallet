// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives

public extension AlertMessage {
    init?(title: String? = nil, error: Error) {
        guard !error.isCancelled else { return nil }
        self.init(title: title, message: error.localizedDescription)
    }
}
