// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import PrimitivesComponents
import Testing

struct AppDisplayFormatterTests {
    @Test
    func format() {
        #expect(AppDisplayFormatter.format(name: "Polymarket", host: "polymarket.com") == "Polymarket (polymarket.com)")

        #expect(AppDisplayFormatter.format(name: "Local App", host: nil) == "Local App")
        #expect(AppDisplayFormatter.format(name: "Test DApp", host: "") == "Test DApp")
        #expect(AppDisplayFormatter.format(name: "My Wallet", host: "   ") == "My Wallet")

        #expect(AppDisplayFormatter.format(name: nil, host: "example.com") == "example.com")
        #expect(AppDisplayFormatter.format(name: "", host: "example.com") == "example.com")
        #expect(AppDisplayFormatter.format(name: "   ", host: "example.com") == "example.com")

        #expect(AppDisplayFormatter.format(name: nil, host: nil) == "Unknown")
        #expect(AppDisplayFormatter.format(name: "", host: "") == "Unknown")
        #expect(AppDisplayFormatter.format(name: "   ", host: "   ") == "Unknown")
    }
}
