// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletConnectServiceProtocol
import enum Gemstone.WalletConnectionVerificationStatus
import Primitives

public struct WalletConnectSessionApproval: Sendable {
    public let chains: [Chain]
    public let accounts: [Primitives.Account]
    public let methods: [String]
    public let events: [String]
}

public extension GemWalletConnectServiceProtocol {
    func metadata(name: String, description: String, url: String, icons: [String]) -> Primitives.ApplicationMetadata {
        applicationMetadata(name: name, description: description, url: url, icons: icons).map()
    }

    func prepareSessionProposal(
        requiredChainIds: [String],
        optionalChainIds: [String],
        metadata: Primitives.ApplicationMetadata,
        origin: String?,
        validation: WalletConnectionVerificationStatus,
    ) async throws -> (proposal: WalletConnectionSessionProposal, verificationStatus: WalletConnectionVerificationStatus) {
        let result = try await prepareSessionProposal(
            requiredChainIds: requiredChainIds,
            optionalChainIds: optionalChainIds,
            metadata: metadata.map(),
            origin: origin,
            validation: validation,
        )
        return (try WalletConnectionSessionProposal(result.proposal), result.verificationStatus)
    }

    func sessionApproval(wallet: Wallet) throws -> WalletConnectSessionApproval {
        let approval = sessionApproval(wallet: wallet.map())
        return WalletConnectSessionApproval(
            chains: approval.chains.map { Primitives.Chain(core: $0) },
            accounts: approval.accounts.map { $0.map() },
            methods: approval.methods,
            events: approval.events,
        )
    }

    func session(topic: String, accounts: [String], expireAt: Date, metadata: Primitives.ApplicationMetadata) throws -> WalletConnectionSession {
        try WalletConnectionSession(session(topic: topic, accounts: accounts, expireAt: Int64(expireAt.timeIntervalSince1970), metadata: metadata.map()))
    }

    func addConnection(_ connection: WalletConnection) async throws {
        try await addConnection(connection: connection.json())
    }

    func updateSessions(_ sessions: [WalletConnectionSession]) async throws {
        try await updateSessions(sessions: sessions.map { $0.json() })
    }
}
