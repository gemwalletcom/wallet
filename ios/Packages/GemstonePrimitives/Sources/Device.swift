// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone

public func generateDeviceKeyPair() -> (privateKey: Data, publicKey: Data) {
    let keyPair = Gemstone.generateDeviceKeyPair()
    return (keyPair.privateKey, keyPair.publicKey)
}

public func devicePublicKey(privateKey: Data) throws -> Data {
    try Gemstone.devicePublicKey(privateKey: privateKey)
}

public func signDeviceAuth(
    privateKey: Data,
    method: String,
    path: String,
    walletId: String,
    body: Data,
    timestampMs: UInt64
) throws -> String {
    try Gemstone.signDeviceAuth(
        privateKey: privateKey,
        method: method,
        path: path,
        walletId: walletId,
        body: body,
        timestampMs: timestampMs
    )
}
