// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import NativeProviderService
import Primitives

public protocol ChainServiceFactorable: Sendable {
    func service(for chain: Chain) -> any ChainServiceable
}

public final class ChainServiceFactory: ChainServiceFactorable, Sendable {
    private let gatewayService: GatewayService
    private let requestInterceptor: any RequestInterceptable

    public init(nodeProvider: any NodeURLFetchable) {
        self.gatewayService = GatewayService(provider: NativeProvider(nodeProvider: nodeProvider))
        self.requestInterceptor = EmptyRequestInterceptor()
    }

    public init(
        gatewayService: GatewayService,
        requestInterceptor: any RequestInterceptable,
    ) {
        self.gatewayService = gatewayService
        self.requestInterceptor = requestInterceptor
    }

    public func service(for chain: Chain) -> any ChainServiceable {
        ChainService.service(chain: chain, gateway: gatewayService)
    }

    public func service(for chain: Chain, url: URL) -> any ChainServiceable {
        ChainService.service(
            chain: chain,
            gateway: GatewayService(provider: NativeProvider(url: url, requestInterceptor: requestInterceptor)),
        )
    }
}
