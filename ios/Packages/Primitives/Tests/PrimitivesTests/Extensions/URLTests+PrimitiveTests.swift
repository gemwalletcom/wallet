// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import Primitives
import Testing

struct URLTests {
    @Test
    func testCleanHost() {
        #expect(URL(string: "https://www.example.com")?.cleanHost() == "example.com")
        #expect(URL(string: "https://www.example.com/about-us")?.cleanHost() == "example.com")
    }
}
