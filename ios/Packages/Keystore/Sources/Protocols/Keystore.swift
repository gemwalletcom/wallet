// Copyright (c). Gem Wallet. All rights reserved.

public import class Gemstone.MessageSigner
import Foundation
import Primitives

internal import SwiftUI

public protocol Keystore: Sendable {
    func createWallet() throws -> [String]
    func previewImport(type: KeystoreImportType) async throws -> WalletImport
    @discardableResult
    func importWallet(name: String, type: KeystoreImportType, isWalletsEmpty: Bool, source: WalletSource) async throws -> Wallet
    func setupChains(chains: [Chain], for wallets: [Wallet]) throws -> [Wallet]
    /// Migrates a v3 keystore to v4; returns the new keystore_id, or nil if already v4.
    func migrateV3Keystore(for wallet: Wallet) async throws -> String?
    func deleteKey(for wallet: Wallet) async throws
    func sign(wallet: Wallet, input: SignerInput) async throws -> [String]
    func signMessage(signer: MessageSigner, wallet: Wallet) async throws -> String
    func signAuthMessageHash(wallet: Wallet, chain: Chain, hash: Data) async throws -> String
    func getPrivateKeyEncoded(wallet: Wallet, chain: Chain) async throws -> String
    func getMnemonic(wallet: Wallet) async throws -> [String]
    func getPasswordAuthentication() throws -> KeystoreAuthentication
    func destroy() throws
}
