// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import struct Gemstone.GemSignerInput
public import protocol Gemstone.GemTransactionSigner
import enum Gemstone.GemstoneError
import GemstonePrimitives
import Primitives

public final class KeystoreTransactionSigner: GemTransactionSigner {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func sign(wallet: String, input: GemSignerInput) async throws -> [GemSignedTransaction] {
        do {
            return try await keystore.sign(wallet: try Primitives.Wallet(wallet), input: input)
        } catch where error.isAuthenticationCancelled {
            throw GemstoneError.Cancelled
        }
    }
}
