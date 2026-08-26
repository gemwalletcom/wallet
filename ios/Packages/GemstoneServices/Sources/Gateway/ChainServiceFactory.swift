// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import NativeProviderService
import Primitives

public protocol ChainServiceFactorable: Sendable {
    func service(for chain: Chain) -> any ChainServiceable
}

public final class ChainServiceFactory: ChainServiceFactorable, Sendable {
    private let gatewayService: GatewayService

    public init(gatewayService: GatewayService) {
        self.gatewayService = gatewayService
    }

    public func service(for chain: Chain) -> any ChainServiceable {
        ChainService.service(chain: chain, gateway: gatewayService)
    }

    public func service(for chain: Chain, url: URL) -> any ChainServiceable {
        ChainService.service(
            chain: chain,
            gateway: gatewayService.with(provider: NativeProvider(url: url)),
        )
    }
}
