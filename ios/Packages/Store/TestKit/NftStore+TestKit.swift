// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Store

public extension NftStore {
    static func mock(db: DB = .mock()) -> NftStore {
        NftStore(db: db)
    }
}
