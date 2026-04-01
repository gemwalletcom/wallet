// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Keystore
import Primitives
import PrimitivesTestKit

public struct KeystoreMock: Keystore {
    public init() {}
    public func createWallet() -> [String] { LocalKeystore.words }
    public func importWallet(name _: String, type _: KeystoreImportType, isWalletsEmpty _: Bool, source _: WalletSource) throws -> Wallet { .mock() }
    public func setupChains(chains _: [Primitives.Chain], for _: [Primitives.Wallet]) throws -> [Wallet] { [.mock()] }
    public func deleteKey(for _: Primitives.Wallet) throws {}
    public func getPrivateKey(wallet _: Primitives.Wallet, chain _: Primitives.Chain) throws -> Data { Data() }
    public func getPrivateKeyEncoded(wallet _: Primitives.Wallet, chain _: Primitives.Chain) throws -> String { .empty }
    public func getMnemonic(wallet _: Primitives.Wallet) throws -> [String] { LocalKeystore.words }
    public func getPasswordAuthentication() throws -> KeystoreAuthentication { .none }
    public func sign(hash _: Data, wallet _: Primitives.Wallet, chain _: Primitives.Chain) throws -> Data { Data() }
    public func destroy() throws {}
}
