// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
@testable import NativeProviderService
import Testing

struct AlienTargetRequestTests {
    @Test
    func requestCarriesTargetHeadersAndBody() throws {
        let target = AlienTarget(
            url: "https://example.com/info",
            method: .post,
            headers: ["Accept": "application/json"],
            body: Data("{}".utf8),
        )

        let request = try target.asRequest()

        #expect(request.url == URL(string: "https://example.com/info"))
        #expect(request.httpMethod == "POST")
        #expect(request.value(forHTTPHeaderField: "Accept") == "application/json")
        #expect(request.httpBody == Data("{}".utf8))
    }

    @Test
    func requestRejectsInvalidURL() {
        let target = AlienTarget(url: "", method: .get, headers: nil, body: nil)

        #expect(throws: AlienError.self) { try target.asRequest() }
    }
}
