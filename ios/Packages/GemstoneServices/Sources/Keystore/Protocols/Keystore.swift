// Copyright (c). Gem Wallet. All rights reserved.

public import struct Gemstone.GemSignedTransaction
public import struct Gemstone.GemSignerInput
public import class Gemstone.MessageSigner
import GemstonePrimitives
import Foundation
import Primitives

internal import SwiftUI

public protocol Keystore: Sendable {
    func keystorePassword(createIfMissing: Bool) throws -> String
    /// Migrates pending v3 keystores to v4, reading the password at most once; returns per-wallet failures.
    func migrateV3Keystores(for wallets: [Wallet]) async throws -> [KeystoreMigrationFailure]
    func sign(wallet: Wallet, input: GemSignerInput) async throws -> [GemSignedTransaction]
    func signMessage(signer: MessageSigner, wallet: Wallet) async throws -> String
    func getPrivateKeyEncoded(wallet: Wallet, chain: Chain) async throws -> String
    func getMnemonic(wallet: Wallet) async throws -> [String]
    func getPasswordAuthentication() throws -> KeystoreAuthentication
    func destroy() throws
}
