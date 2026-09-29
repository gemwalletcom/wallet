// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing

struct JSONDateDecoderTests {
    struct TestDate: Codable, Equatable {
        let date: Date
    }

    @Test
    func decodesFractionalAndWholeSeconds() throws {
        _ = try JSONDateDecoder.standard.decode(TestDate.self, from: Data("{\"date\": \"2023-12-26T21:47:58.101180Z\"}".utf8))
        _ = try JSONDateDecoder.standard.decode(TestDate.self, from: Data("{\"date\": \"2025-04-09T17:30:40Z\"}".utf8))
    }

    @Test
    func decodesZeroNanoseconds() throws {
        let decoded = try JSONDateDecoder.standard.decode(TestDate.self, from: Data("{\"date\": \"2023-12-26T21:47:40Z\"}".utf8))
        #expect(decoded == TestDate(date: Date(timeIntervalSince1970: 1_703_627_260)))
    }
}
