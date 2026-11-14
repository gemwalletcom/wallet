// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import protocol Gemstone.GemKeystorePassword
import enum Gemstone.GemServiceError
import typealias Gemstone.WalletId
import Primitives

public final class GemstoneKeystorePassword: GemKeystorePassword, Sendable {
    private let password: any KeystorePassword

    public init(password: any KeystorePassword) {
        self.password = password
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        do {
            return try password.getPassword(createIfMissing: createIfMissing)
        } catch where error.isAuthenticationCancelled {
            throw GemServiceError.Cancelled
        }
    }

    public func getWalletPassword(walletId _: Gemstone.WalletId) throws -> String? {
        nil
    }

    public func deleteWalletPassword(walletId _: Gemstone.WalletId) throws {}

    public func authentication() throws -> GemKeystoreAuthentication {
        try password.getAuthentication()
    }
}
