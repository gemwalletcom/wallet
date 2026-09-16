// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension AssetsRequest {
    static func mock(
        scope: AssetsRequestScope = .wallet,
        searchBy: String = "",
        filters: [AssetsRequestFilter] = [],
    ) -> AssetsRequest {
        AssetsRequest(walletId: .mock(), scope: scope, searchBy: searchBy, filters: filters)
    }
}
