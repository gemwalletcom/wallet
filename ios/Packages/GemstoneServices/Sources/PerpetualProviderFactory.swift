// Copyright (c). Gem Wallet. All rights reserved.

import Blockchain
import Foundation
import NativeProviderService
import Primitives

public struct PerpetualProviderFactory {
    private let nodeProvider: any NodeURLFetchable
    private let requestInterceptor: any RequestInterceptable

    public init(
        nodeProvider: any NodeURLFetchable,
        requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor(),
    ) {
        self.nodeProvider = nodeProvider
        self.requestInterceptor = requestInterceptor
    }

    public func createProvider(chain: Chain = .hyperCore) -> PerpetualProvidable {
        GatewayPerpetualProvider(
            gateway: GatewayService(
                provider: NativeProvider(nodeProvider: nodeProvider, requestInterceptor: requestInterceptor),
            ),
            chain: chain,
        )
    }
}
