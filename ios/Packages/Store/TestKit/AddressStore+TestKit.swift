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

public extension AddressNameUpdate {
    static func mock(_ name: AddressName, replaces: [AddressType] = [.address, .contract, .validator]) -> AddressNameUpdate {
        AddressNameUpdate(name: name, replacesTypes: replaces.contains(name.type) ? replaces : replaces + [name.type])
    }
}
