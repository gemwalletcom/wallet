// Copyright (c). Gem Wallet. All rights reserved.

public import class Gemstone.MessageSigner
import Foundation
import Keystore
import Primitives
import PrimitivesTestKit

public struct KeystoreMock: Keystore {
    public init() {}
    public func createWallet() -> [String] {
        LocalKeystore.words
    }

    public func previewImport(type _: KeystoreImportType) -> WalletImport {
        WalletImport(walletId: .mock(), walletType: .multicoin, accounts: [])
    }

    public func importWallet(name _: String, type _: KeystoreImportType, isWalletsEmpty _: Bool, source _: WalletSource) throws -> Wallet {
        .mock()
    }

    public func setupChains(chains _: [Primitives.Chain], for _: [Primitives.Wallet]) throws -> [Wallet] {
        [.mock()]
    }

    public func migrateV3Keystores(for _: [Primitives.Wallet]) throws -> [KeystoreMigrationFailure] {
        []
    }

    public func deleteKey(for _: Primitives.Wallet) throws {}

    public func sign(wallet _: Primitives.Wallet, input _: SignerInput) throws -> [String] {
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
