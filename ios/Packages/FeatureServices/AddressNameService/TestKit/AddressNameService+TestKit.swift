// Copyright (c). Gem Wallet. All rights reserved.

import AddressNameService
import Foundation
import protocol Gemstone.GemNameServiceProtocol
import GemstonePrimitivesTestKit
import Store
import StoreTestKit

public extension AddressNameService {
    static func mock(
        addressStore: AddressStore = .mock(),
        apiService: any GemNameServiceProtocol = GemNameServiceMock(),
    ) -> AddressNameService {
        AddressNameService(addressStore: addressStore, apiService: apiService)
    }
}
