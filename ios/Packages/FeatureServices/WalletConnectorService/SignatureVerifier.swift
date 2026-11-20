// Copyright (c). Gem Wallet. All rights reserved.

import CryptoKit
import Foundation
import protocol Gemstone.GemSignatureVerifier

public final class SignatureVerifier: Sendable {
    public init() {}
}

extension SignatureVerifier: GemSignatureVerifier {
    public func isValidEs256(publicKey: Data, message: Data, signature: Data) -> Bool {
        guard let key = try? P256.Signing.PublicKey(x963Representation: publicKey),
              let signature = try? P256.Signing.ECDSASignature(rawRepresentation: signature)
        else {
            return false
        }
        return key.isValidSignature(signature, for: SHA256.hash(data: message))
    }
}
