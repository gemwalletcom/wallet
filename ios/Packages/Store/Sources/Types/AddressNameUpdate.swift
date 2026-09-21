// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public struct AddressNameUpdate: Sendable {
    public let name: AddressName
    public let replacesTypes: [AddressType]

    public init(name: AddressName, replacesTypes: [AddressType]) {
        self.name = name
        self.replacesTypes = replacesTypes
    }
}
