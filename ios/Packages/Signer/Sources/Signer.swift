// Copyright (c). Gem Wallet. All rights reserved.

internal import class Gemstone.MessageSigner
internal import struct Gemstone.SignMessage
import Foundation
import Keystore
import Primitives

public struct Signer: Sendable {
    let wallet: Primitives.Wallet
    let keystore: any Keystore

    public init(
        wallet: Primitives.Wallet,
        keystore: any Keystore,
    ) {
        self.wallet = wallet
        self.keystore = keystore
    }

    public func signTypedMessage(chain: Primitives.Chain, message: String) async throws -> String {
        let messageSigner = MessageSigner(message: SignMessage(chain: chain.rawValue, signType: .eip712, data: Data(message.utf8)))
        return try await keystore.signMessage(signer: messageSigner, wallet: wallet)
    }
}
