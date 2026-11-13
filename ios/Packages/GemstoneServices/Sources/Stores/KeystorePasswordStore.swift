// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import protocol Gemstone.GemKeystorePassword
import enum Gemstone.GemServiceError
import typealias Gemstone.WalletId
import Primitives

public final class GemstoneKeystorePassword: GemKeystorePassword, Sendable {
    private let password: any KeystorePassword
    private let unlockContext: UnlockContextStore?

    public init(password: any KeystorePassword, unlockContext: UnlockContextStore? = nil) {
        self.password = password
        self.unlockContext = unlockContext
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        guard let unlockContext, try password.getAuthentication() != .none else {
            do {
                return try password.getPassword(createIfMissing: createIfMissing)
            } catch where error.isAuthenticationCancelled {
                throw GemServiceError.Cancelled
            }
        }
        guard let context = unlockContext.current else {
            throw GemServiceError.Cancelled
        }
        context.interactionNotAllowed = true
        do {
            return try password.getPassword(context: context)
        } catch where error.isAuthenticationCancelled || error.isInteractionNotAllowed {
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
