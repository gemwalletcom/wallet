// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import class Gemstone.MessageSigner
public import GemstonePrimitives
import Foundation
import Keystore
import Primitives
import PrimitivesTestKit

public struct KeystoreMock: Keystore {
    public init() {}

    public func keystorePassword(createIfMissing _: Bool) throws -> Data {
        Data()
    }

    public func migrateV3Keystores(for _: [Primitives.Wallet]) throws -> [KeystoreMigrationFailure] {
        []
    }

    public func deleteKey(for _: Primitives.Wallet) throws {}

    public func sign(wallet _: Primitives.Wallet, input _: SignerInput) throws -> [GemSignedTransaction] {
        []
    }

    public func signMessage(signer _: MessageSigner, wallet _: Primitives.Wallet) throws -> String {
        .empty
    }

    public func signAuthMessageHash(wallet _: Primitives.Wallet, chain _: Primitives.Chain, hash _: Data) throws -> String {
        .empty
    }

    public func getPrivateKeyEncoded(wallet _: Primitives.Wallet, chain _: Primitives.Chain) throws -> String {
        .empty
    }

    public func getMnemonic(wallet _: Primitives.Wallet) throws -> [String] {
        LocalKeystore.words
    }

    public func getPasswordAuthentication() throws -> KeystoreAuthentication {
        .none
    }

    public func destroy() throws {}
}
