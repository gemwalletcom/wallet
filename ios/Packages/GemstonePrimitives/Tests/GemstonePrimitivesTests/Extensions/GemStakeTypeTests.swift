// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import GemstonePrimitives
import Primitives
import Testing

final class GemStakeTypeTests {
    @Test
    func freezeMapsToGemStakeType() {
        let mapped = Primitives.StakeType.freeze(.bandwidth).map()

        #expect(mapped == GemStakeType.freeze(resource: .bandwidth))
    }

    @Test
    func unfreezeMapsToGemStakeType() {
        let mapped = Primitives.StakeType.unfreeze(.energy).map()

        #expect(mapped == GemStakeType.unfreeze(resource: .energy))
    }

    @Test
    func unfreezeMapsToPrimitiveStakeType() throws {
        let mapped = try GemStakeType.unfreeze(resource: .energy).map()

        #expect(mapped == Primitives.StakeType.unfreeze(.energy))
    }
}
