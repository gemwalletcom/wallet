// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
@testable import GemTest
import Testing

struct GemTestTests {
    @Test
    func testCache() async throws {
        let cache = Cache<AlienTarget, Data>()
        let target = AlienTarget(
            url: "https://example.com",
            method: .get,
            headers: .none,
            body: .none
        )
        let data = try #require(Data(hex: "0xdeadbeef"))

        await cache.set(value: data, forKey: target, ttl: 1)
        let value = await cache.get(key: target)

        #expect(value == data)

        try await Task.sleep(nanoseconds: 1_100_000_000)
        let expiredValue = await cache.get(key: target)

        #expect(expiredValue == nil)
    }

    @Test
    func testMessagePreview() async throws {
        let base58 = try #require("jo91waLQA1NNeBmZKUF".data(using: .utf8))
        let message = SignMessage(chain: "solana", signType: .base58, data: base58)
        let signer = MessageSigner(message: message)
        let preview = try signer.preview()

        switch preview {
        case .text(let text):
            #expect(text == "this is a test")
        case .eip712, .siwe:
            Issue.record("Unexpected result")
        }

        let result = signer.getResult(
            data: try #require(Data(hex: "7468697320697320612074657374"))
        )
        #expect(result == "jo91waLQA1NNeBmZKUF")
    }

    @Test
    func testMessageHash() async throws {
        let message = SignMessage(
            chain: "ethereum",
            signType: .eip191,
            data: "hello world".data(using: .utf8)!
        )
        let signer = MessageSigner(message: message)
        let hash = try signer.hash()

        #expect(hash.hexString() == "d9eba16ed0ecae432b71fe008c98cc872bb4cc214d3220a36f365326cf807d68")
    }

}
