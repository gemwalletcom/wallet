// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
@testable import NativeProviderService
import Testing

struct NativeProviderCacheTests {
    @Test
    func requestStripsPrivateCacheHeader() throws {
        let target = AlienTarget(
            url: "https://example.com/info",
            method: .get,
            headers: [
                "Accept": "application/json",
                nativeProviderCacheHeader: "60",
            ],
            body: nil,
        )

        let request = try target.asRequest()

        #expect(request.value(forHTTPHeaderField: "Accept") == "application/json")
        #expect(request.value(forHTTPHeaderField: nativeProviderCacheHeader) == nil)
    }
}
