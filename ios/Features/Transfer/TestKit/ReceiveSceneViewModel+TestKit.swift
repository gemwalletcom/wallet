// Copyright (c). Gem Wallet. All rights reserved.

import protocol Gemstone.GemReceiveServiceProtocol
import GemstonePrimitivesTestKit
import Primitives
import PrimitivesTestKit
import Transfer

public extension ReceiveSceneViewModel {
    static func mock(
        service: any GemReceiveServiceProtocol = GemReceiveServiceMock(),
    ) -> ReceiveSceneViewModel {
        ReceiveSceneViewModel(
            assetAddress: AssetAddress(asset: .mock(id: .mock(chain: .bitcoin)), address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"),
            wallet: .mock(accounts: [.mock(chain: .bitcoin, address: "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"), .mock(chain: .ethereum, address: "0xabc"), .mock(chain: .solana, address: "So1ana")]),
            service: service,
        )
    }
}
