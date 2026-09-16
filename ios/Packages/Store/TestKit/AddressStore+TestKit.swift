// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesTestKit
import Store

public extension AddressStore {
    static func mock(db: DB = .mock()) -> Self {
        AddressStore(db: db)
    }
}
