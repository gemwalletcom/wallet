// Copyright (c). Gem Wallet. All rights reserved.

import CryptoKit
import Foundation
import GemstonePrimitives
@testable import GemstoneServices
import Primitives
import Testing

struct DeviceRequestSignerTests {
    @Test
    func signerInitFromPrivateKey() throws {
        let keyPair = generateDeviceKeyPair()
        let signer = try DeviceRequestSigner(privateKey: keyPair.privateKey)

        #expect(signer.publicKeyHex == keyPair.publicKey.hex)
    }

    @Test
    func signerInitFromPrivateKeyHex() throws {
        let keyPair = generateDeviceKeyPair()
        let signer = try DeviceRequestSigner(privateKeyHex: keyPair.privateKey.hex)

        #expect(signer.publicKeyHex == keyPair.publicKey.hex)
    }

    @Test
    func signerRejectsInvalidHex() {
        #expect(throws: (any Error).self) {
            try DeviceRequestSigner(privateKeyHex: "not_valid_hex")
        }
    }

    @Test
    func signatureVerifiesWithPublicKey() throws {
        let keyPair = generateDeviceKeyPair()
        let signer = try DeviceRequestSigner(privateKey: keyPair.privateKey)
        var request = try URLRequest(url: #require(URL(string: "https://api.gemwallet.com/v2/devices")))
        request.httpMethod = "GET"

        try signer.sign(request: &request)

        let parts = try decodePayload(from: request)
        #expect(parts.count == 5)
        #expect(parts[0] == keyPair.publicKey.hex)
        #expect(parts[2] == "")
        let message = "\(parts[1]).GET./v2/devices.\(parts[2]).\(parts[3])"
        let publicKey = try Curve25519.Signing.PublicKey(rawRepresentation: keyPair.publicKey)
        #expect(try publicKey.isValidSignature(Data.from(hex: parts[4]), for: Data(message.utf8)))
    }

    @Test
    func signWithWalletId() throws {
        let keyPair = generateDeviceKeyPair()
        let signer = try DeviceRequestSigner(privateKey: keyPair.privateKey)
        var request = try URLRequest(url: #require(URL(string: "https://api.gemwallet.com/v2/devices/rewards")))
        request.httpMethod = "GET"

        try signer.sign(request: &request, walletId: "multicoin_0xabc")

        let parts = try decodePayload(from: request)
        #expect(parts[2] == "multicoin_0xabc")
        let message = "\(parts[1]).GET./v2/devices/rewards.\(parts[2]).\(parts[3])"
        let publicKey = try Curve25519.Signing.PublicKey(rawRepresentation: Data.from(hex: parts[0]))
        #expect(try publicKey.isValidSignature(Data.from(hex: parts[4]), for: Data(message.utf8)))
    }

    @Test
    func signWithBody() throws {
        let signer = try DeviceRequestSigner(privateKey: generateDeviceKeyPair().privateKey)
        var request = try URLRequest(url: #require(URL(string: "https://api.gemwallet.com/v2/devices")))
        request.httpMethod = "POST"
        request.httpBody = Data("{\"test\":true}".utf8)

        try signer.sign(request: &request)

        let parts = try decodePayload(from: request)
        let emptyBodyHash = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        #expect(parts[3] != emptyBodyHash)
        #expect(parts[3].count == 64)
    }

    private func decodePayload(from request: URLRequest) throws -> [String] {
        let auth = try #require(request.value(forHTTPHeaderField: "Authorization"))
        #expect(auth.hasPrefix("Gem "))
        let data = try #require(Data(base64Encoded: String(auth.dropFirst("Gem ".count))))
        let decoded = try #require(String(data: data, encoding: .utf8))
        return decoded.split(separator: ".", maxSplits: 4, omittingEmptySubsequences: false).map(String.init)
    }
}
