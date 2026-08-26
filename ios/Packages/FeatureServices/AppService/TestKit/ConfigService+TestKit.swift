// Copyright (c). Gem Wallet. All rights reserved.

import AppService
import Foundation
import protocol Gemstone.GemConfigServiceProtocol
import GemstonePrimitivesTestKit
import PrimitivesTestKit

public extension ConfigService {
    static func mock(service: any GemConfigServiceProtocol = GemConfigServiceMock(config: .mock())) -> ConfigService {
        ConfigService(service: service)
    }
}
