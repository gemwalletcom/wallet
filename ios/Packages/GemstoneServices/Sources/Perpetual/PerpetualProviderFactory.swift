// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import NativeProviderService
import Primitives

public struct PerpetualProviderFactory {
    private let gatewayService: GatewayService
    private let nodeProvider: any NodeURLFetchable
    private let requestInterceptor: any RequestInterceptable

    public init(
        gatewayService: GatewayService,
        nodeProvider: any NodeURLFetchable,
        requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor(),
    ) {
        self.gatewayService = gatewayService
        self.nodeProvider = nodeProvider
        self.requestInterceptor = requestInterceptor
    }

    public func createProvider(chain: Chain = .hyperCore) -> PerpetualProvidable {
        GatewayPerpetualProvider(
            gateway: gatewayService.with(provider: NativeProvider(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor)),
            chain: chain,
        )
    }
}
