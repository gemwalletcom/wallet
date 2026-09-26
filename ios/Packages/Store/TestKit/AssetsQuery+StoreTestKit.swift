// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension AssetsQuery {
    static func mock(
        scope: AssetsQueryScope = .wallet,
        searchBy: String = "",
        filters: [AssetsQueryFilter] = [],
    ) -> AssetsQuery {
        AssetsQuery(walletId: .mock(), scope: scope, searchBy: searchBy, filters: filters)
    }
}
