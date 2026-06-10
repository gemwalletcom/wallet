// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public struct ChainService {
    public static func service(chain: Chain, gateway: GatewayService) -> ChainServiceable {
        GatewayChainService(
            chain: chain,
            gateway: gateway,
        )
    }
}
