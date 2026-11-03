// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol LimitedQuery: DatabaseQueryable {
    func with(limit: Int) -> Self
}
