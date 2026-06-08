// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import GemstonePrimitives
import Primitives

public struct DeviceRequestSigner: Sendable {
    private let privateKey: Data
    public let publicKeyHex: String

    public init(privateKey: Data) throws {
        self.privateKey = privateKey
        publicKeyHex = try devicePublicKey(privateKey: privateKey).hex
    }

    public init(privateKeyHex: String) throws {
        try self.init(privateKey: Data.from(hex: privateKeyHex))
    }

    public func sign(request: inout URLRequest, walletId: String = "") throws {
        let method = request.httpMethod ?? "GET"
        let path = request.url?.path ?? "/"
        let body = request.httpBody ?? Data()
        let timestamp = UInt64(Date().timeIntervalSince1970 * 1000)
        let header = try signDeviceAuth(
            privateKey: privateKey,
            method: method,
            path: path,
            walletId: walletId,
            body: body,
            timestampMs: timestamp,
        )
        request.setValue(header, forHTTPHeaderField: "Authorization")
    }
}
