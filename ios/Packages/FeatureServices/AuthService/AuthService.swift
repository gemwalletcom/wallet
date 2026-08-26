// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemAuthServiceProtocol
import func Gemstone.createAuthMessage
import GemstonePrimitives
import Keystore
import Preferences
import Primitives

public protocol AuthServiceable: Sendable {
    func getAuthPayload(wallet: Wallet) async throws -> AuthPayload
}

public struct AuthService: AuthServiceable, Sendable {
    private let apiService: any GemAuthServiceProtocol
    private let keystore: any Keystore
    private let securePreferences: SecurePreferences

    public init(
        apiService: any GemAuthServiceProtocol,
        keystore: any Keystore,
        securePreferences: SecurePreferences = SecurePreferences(),
    ) {
        self.apiService = apiService
        self.keystore = keystore
        self.securePreferences = securePreferences
    }

    public func getAuthPayload(wallet: Wallet) async throws -> AuthPayload {
        let deviceId = try securePreferences.getDeviceId()
        let chain = Chain.ethereum
        let account = try wallet.account(for: chain)

        let nonce = try await apiService.getNonce()
        let authNonce = try AuthNonce(nonce)
        let authMessage = Gemstone.createAuthMessage(address: account.address, authNonce: nonce)
        let signature = try await keystore.signAuthMessageHash(wallet: wallet, chain: chain, hash: Data(authMessage.hash))

        return AuthPayload(
            deviceId: deviceId,
            chain: chain,
            address: account.address,
            nonce: authNonce.nonce,
            signature: signature,
        )
    }
}
