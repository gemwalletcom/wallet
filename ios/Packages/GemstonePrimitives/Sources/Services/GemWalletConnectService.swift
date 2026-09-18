// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import protocol Gemstone.GemWalletConnectServiceProtocol
import enum Gemstone.WalletConnectionVerificationStatus
import Primitives

public extension GemWalletConnectServiceProtocol {
    func metadata(name: String, description: String, url: String, icons: [String]) -> Primitives.ApplicationMetadata {
        applicationMetadata(name: name, description: description, url: url, icons: icons).toPrimitives()
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
            metadata: metadata.toGem(),
            origin: origin,
            validation: validation,
        )
        return (result.proposal.toPrimitives(), result.verificationStatus)
    }

    func session(topic: String, accounts: [String], expireAt: Date, metadata: Primitives.ApplicationMetadata) throws -> WalletConnectionSession {
        try session(topic: topic, accounts: accounts, expireAt: Int64(expireAt.timeIntervalSince1970), metadata: metadata.toGem()).toPrimitives()
    }

    func addConnection(_ connection: WalletConnection) async throws {
        try await addConnection(connection: connection.toGem())
    }

    func updateSessions(_ sessions: [WalletConnectionSession]) async throws {
        try await updateSessions(sessions: sessions.map { $0.toGem() })
    }
}
