// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives

internal import Gemstone

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

    public func sign(input: SignerInput) async throws -> [String] {
        try await keystore.sign(wallet: wallet, input: input)
    }

    public func signTypedMessage(chain: Primitives.Chain, message: String) async throws -> String {
        let messageSigner = Gemstone.MessageSigner(message: Gemstone.SignMessage(chain: chain.rawValue, signType: .eip712, data: Data(message.utf8)))
        return try await keystore.signMessage(signer: messageSigner, wallet: wallet)
    }
}
