// Copyright (c). Gem Wallet. All rights reserved.

import ChainService
import Foundation

public extension AddressStatusService {
    static func mock(
        chainServiceFactory: any ChainServiceFactorable = ChainServiceFactoryMock(),
    ) -> AddressStatusService {
        AddressStatusService(chainServiceFactory: chainServiceFactory)
    }
}
