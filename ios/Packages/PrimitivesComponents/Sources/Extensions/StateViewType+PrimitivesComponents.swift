// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives

public extension StateViewType {
    mutating func setError(_ error: Error) {
        guard !error.isCancelled else { return }
        self = .error(error)
    }
}
