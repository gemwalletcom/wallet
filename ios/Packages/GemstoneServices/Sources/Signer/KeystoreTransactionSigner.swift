// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import struct Gemstone.GemSignerInput
public import protocol Gemstone.GemTransactionSigner
public import struct Gemstone.Wallet
import enum Gemstone.GemstoneError
import GemstonePrimitives
import Primitives

public final class KeystoreTransactionSigner: GemTransactionSigner {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func sign(wallet: Gemstone.Wallet, input: GemSignerInput) async throws -> [GemSignedTransaction] {
        do {
            return try await keystore.sign(wallet: wallet.map(), input: input)
        } catch where error.isAuthenticationCancelled {
            throw GemstoneError.Cancelled
        }
    }
}
