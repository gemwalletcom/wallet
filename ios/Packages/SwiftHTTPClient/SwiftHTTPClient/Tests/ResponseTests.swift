// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient
import Testing

struct ResponseTests {
    @Test
    func mapInt() throws {
        let response = Response(body: Data("1".utf8))
        #expect(try response.map(as: Int.self, decoder: JSONDecoder()) == 1)
    }

    @Test
    func mapArray() throws {
        let response = Response(body: Data("[]".utf8))
        #expect(try response.map(as: [String].self, decoder: JSONDecoder()) == [])
    }
}
