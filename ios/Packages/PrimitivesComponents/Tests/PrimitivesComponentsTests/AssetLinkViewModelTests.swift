// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesTestKit
import Testing

@testable import PrimitivesComponents

struct AssetLinkViewModelTests {
    @Test
    func testHost() {
        #expect(AssetLinkViewModel(.mock(type: .website)).host == "example.com")
        #expect(AssetLinkViewModel(.mock(type: .gitHub)).host == .none)
    }
}
