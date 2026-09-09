// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsNestedRecord() throws {
        let config = ConfigResponse.mock(versions: .mock(swapAssets: 7))
        let decoded = try Primitives.ConfigResponse(config.json())

        #expect(decoded.versions.swapAssets == 7)
        #expect(decoded.releases.map(\.version) == config.releases.map(\.version))
    }
}
