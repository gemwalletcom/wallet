// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemImportType
import struct Gemstone.GemKeystoreAccount
import struct Gemstone.GemStoredWallet
import enum Gemstone.GemWalletImportType
import GemstonePrimitives
import GemstoneServices
import Primitives

public struct LocalKeystoreMockContext {
    public let keystore: LocalKeystore
    public let baseDir: URL
    public let password: MockKeystorePassword
}

/// For public use
public extension LocalKeystore {
    static let words = ["shoot", "island", "position", "soft", "burden", "budget", "tooth", "cruel", "issue", "economy", "destroy", "above"]
    static let privateKey = "0x9f110a73d04dc7becb316fb9adfe04689a947bb49be11060577c3c0a4b4d4cd5"
    static let address = "0x734dC149D4c7D0D5E95B5AA787e5FB288dD167a9"
    static let bitcoinAddress = "bc1quvuarfksewfeuevuc6tn0kfyptgjvwsvrprk9d"
    static let password = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f"

    static func mock(
        keystorePassword: KeystorePassword = MockKeystorePassword(),
    ) -> LocalKeystore {
        LocalKeystore(
            directory: UUID().uuidString,
            keystorePassword: keystorePassword,
        )
    }

    static func mockContext(
        keystorePassword: MockKeystorePassword = MockKeystorePassword(memoryPassword: LocalKeystore.password),
    ) throws -> LocalKeystoreMockContext {
        let directory = UUID().uuidString
        let baseDir = try FileManager.default
            .url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true)
            .appending(path: directory, directoryHint: .isDirectory)
        let keystore = LocalKeystore(
            directory: directory,
            keystorePassword: keystorePassword,
        )
        return LocalKeystoreMockContext(
            keystore: keystore,
            baseDir: baseDir,
            password: keystorePassword,
        )
    }
}

public extension LocalKeystore {
    func importWallet(name: String, type: GemWalletImportType) throws -> Primitives.Wallet {
        switch type {
        case let .address(address, chain):
            return viewWallet(name: name, chain: Primitives.Chain(core: chain), address: address)
        case let .multicoinPhrase(words, chains):
            return try importWallet(name: name, import: .multicoinPhrase(words: words, chains: chains))
        case let .singlePhrase(words, chain):
            return try importWallet(name: name, import: .singlePhrase(words: words, chain: chain))
        case let .privateKey(value, chain):
            return try importWallet(name: name, import: .privateKey(value: value, chain: chain))
        }
    }

    private func importWallet(name: String, import: GemImportType) throws -> Primitives.Wallet {
        let password = try keystorePassword(createIfMissing: true)
        return try gemKeystore.createStore(import: `import`, password: gemKeystore.decodePassword(password: password)).mapToWallet(name: name, source: .import)
    }
}

private func viewWallet(name: String, chain: Chain, address: String) -> Primitives.Wallet {
    Primitives.Wallet(
        id: WalletId.make(walletType: .view, chain: chain, address: address),
        externalId: nil,
        name: name,
        index: 0,
        type: .view,
        accounts: [Account(chain: chain, address: address, derivationPath: "", extendedPublicKey: "")],
        isPinned: false,
        imageUrl: nil,
        source: .import,
    )
}

private extension GemKeystoreAccount {
    func mapToAccount() -> Primitives.Account {
        Primitives.Account(
            chain: Primitives.Chain(core: chain),
            address: address,
            derivationPath: derivationPath,
            extendedPublicKey: publicKey ?? "",
        )
    }
}

private extension GemStoredWallet {
    func mapToWallet(name: String, source: Primitives.WalletSource) throws -> Primitives.Wallet {
        try Primitives.Wallet(
            id: Primitives.WalletId.from(id: walletId),
            externalId: nil,
            name: name,
            index: 0,
            type: walletType.map(),
            accounts: accounts.map { $0.mapToAccount() },
            isPinned: false,
            imageUrl: nil,
            source: source,
        )
    }
}
