// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.GemWalletConnectAuthMessage
import Primitives

public struct WCAuthenticationRequest: Sendable {
    public let id: String
    public let proposal: WalletConnectionSessionProposal
    public let verificationStatus: WalletConnectionVerificationStatus
    public let message: @Sendable (Wallet) throws -> GemWalletConnectAuthMessage
    public let approve: @Sendable (Wallet, String) async throws -> Void

    public init(
        id: String,
        proposal: WalletConnectionSessionProposal,
        verificationStatus: WalletConnectionVerificationStatus,
        message: @escaping @Sendable (Wallet) throws -> GemWalletConnectAuthMessage,
        approve: @escaping @Sendable (Wallet, String) async throws -> Void,
    ) {
        self.id = id
        self.proposal = proposal
        self.verificationStatus = verificationStatus
        self.message = message
        self.approve = approve
    }
}
