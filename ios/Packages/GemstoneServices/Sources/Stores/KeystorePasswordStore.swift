// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.GemKeystoreAuthentication
import protocol Gemstone.GemKeystorePassword
import typealias Gemstone.WalletId
import enum Gemstone.GemServiceError
import Keychain

public final class GemstoneKeystorePassword: GemKeystorePassword, @unchecked Sendable {
    private let keystore: any Keystore

    public init(keystore: any Keystore) {
        self.keystore = keystore
    }

    public func getPassword(createIfMissing: Bool) throws -> String {
        do {
            return try keystore.keystorePassword(createIfMissing: createIfMissing)
        } catch where error.isKeychainUserCancelled {
            throw GemServiceError.Cancelled
        }
    }

    public func getWalletPassword(walletId _: Gemstone.WalletId) throws -> String? {
        nil
    }

    public func deleteWalletPassword(walletId _: Gemstone.WalletId) throws {}

    public func authentication() throws -> GemKeystoreAuthentication {
        switch try keystore.getPasswordAuthentication() {
        case .biometrics: .biometrics
        case .passcode: .passcode
        case .none: .none
        }
    }
}
