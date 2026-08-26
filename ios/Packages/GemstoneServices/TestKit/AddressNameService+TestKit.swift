// Copyright (c). Gem Wallet. All rights reserved.

import GemstoneServices
import Foundation
import PrimitivesTestKit
import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit

public extension AddressNameService {
    static func mock(
        addressStore: AddressStore = .mock(),
        service: any GemNameServiceProtocol = GemNameServiceMock(),
    ) -> AddressNameService {
        AddressNameService(addressStore: addressStore, service: service)
    }
}
