// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import NativeProviderService
import Primitives
import ScanService

public extension ScanService {
    static func mock() -> ScanService {
        let mockProvider = NativeProvider(url: Constants.apiURL, requestInterceptor: EmptyRequestInterceptor())
        let gatewayService = GatewayService(provider: mockProvider)
        return ScanService(gatewayService: gatewayService)
    }
}
