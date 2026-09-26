// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import SwiftHTTPClient
import Testing

struct TestDate: Codable, Equatable {
    let date: Date
}

struct ResponseTests {
    @Test
    func mapInt() throws {
        let response = Response(body: Data("1".utf8))
        #expect(try response.map(as: Int.self) == 1)
    }

    @Test
    func mapArray() throws {
        let response = Response(body: Data("[]".utf8))
        #expect(try response.map(as: [String].self) == [])
    }

    @Test
    func mapDate() throws {
        _ = try Response(body: Data("{\"date\": \"2023-12-26T21:47:58.101180Z\"}".utf8)).map(as: TestDate.self)
        _ = try Response(body: Data("{\"date\": \"2023-12-26T21:47:58.101180Z\"}".utf8)).map(as: TestDate.self)
        _ = try Response(body: Data("{\"date\": \"2025-04-09T17:30:40Z\"}".utf8)).map(as: TestDate.self)
    }

    @Test
    func mapDateZeroNanoseconds() throws {
        let response = Response(body: Data("{\"date\": \"2023-12-26T21:47:40Z\"}".utf8))
        let expected = TestDate(date: Date(timeIntervalSince1970: 1_703_627_260))
        #expect(try response.map(as: TestDate.self) == expected)
    }
}
