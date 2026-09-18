// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemAddressService
import class Gemstone.MessageSigner
import GemstonePrimitives
import struct Gemstone.SignMessage
import Primitives
import Testing

struct MessageSignerTests {
    @Test
    func eip191SiweUsesPayloadPreview() throws {
        let message = """
        thepoc.xyz wants you to sign in with your Ethereum account:
        0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4

        Sign in with different chain ID

        URI: https://thepoc.xyz
        Version: 1
        Chain ID: 1
        Nonce: byjof9dwrao97skautdxhb
        Issued At: 2026-03-09T15:48:34.458Z
        """

        let signer = MessageSigner(
            message: SignMessage(chain: "ethereum", signType: .eip191, data: Data(message.utf8)),
        )

        let preview = try signer.payloadPreview(simulationPayload: [])
        let primary = preview?.primary

        #expect(primary?.map(\.value) == [
            .text(text: "thepoc.xyz"),
            .address(
                display: GemAddressService.shared.format(address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4", chain: .ethereum),
                address: "0xBA4D1d35bCe0e8F28E5a3403e7a0b996c5d50AC4"
            ),
        ])
        #expect(preview?.secondary.count == 5)
    }
}
