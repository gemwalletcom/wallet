// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
@testable import PrimitivesComponents
import PrimitivesTestKit
import Testing

struct AssetLinkViewModelTests {
    @Test
    func testHost() {
        #expect(AssetLinkViewModel(.mock(type: .website)).host == "example.com")
        #expect(AssetLinkViewModel(.mock(type: .gitHub)).host == .none)
    }
}
