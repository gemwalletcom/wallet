// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
@testable import GemstonePrimitives
import Primitives
import PrimitivesTestKit
import Testing

struct JsonCodableTests {
    @Test
    func roundTripsTaggedEnum() throws {
        let stakeType = Primitives.StakeType.freeze(.bandwidth)

        #expect(try Primitives.StakeType(stakeType.json()) == stakeType)
    }

    @Test
    func roundTripsDate() throws {
        let message = Primitives.SupportMessage.mock(createdAt: Date(timeIntervalSince1970: 1_700_000_000))

        #expect(try Primitives.SupportMessage(message.json()).createdAt == message.createdAt)
    }

    @Test
    func roundTripsNestedRecord() throws {
        let images = Primitives.NFTImages(preview: Primitives.NFTResource(url: "https://example.com/a.png", mimeType: "image/png"))
        let decoded = try Primitives.NFTImages(images.json())
        #expect(decoded.preview.url == images.preview.url)
        #expect(decoded.preview.mimeType == images.preview.mimeType)
    }
}
