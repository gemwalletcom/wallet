// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitivesTestKit
import Primitives
import PrimitivesComponents
import PrimitivesTestKit

public extension AddressInputViewModel {
    static func mock() -> AddressInputViewModel {
        AddressInputViewModel(chain: .ethereum, nameService: GemNameServiceMock(nameRecord: .mock()), placeholder: "Address")
    }
}
