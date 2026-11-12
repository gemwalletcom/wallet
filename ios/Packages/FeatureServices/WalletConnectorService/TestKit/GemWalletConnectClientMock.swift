// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import GemstonePrimitivesTestKit

public actor GemWalletConnectClientMock: GemWalletConnectClientProtocol {
    public private(set) var pairedURIs: [String] = []

    public init() {}

    public nonisolated func needsConnection() throws -> Bool {
        true
    }

    public func connected() async throws {}

    public func restore() async throws {}

    public func receive(frame _: String) async throws {}

    public func pair(uri: String) async throws {
        pairedURIs.append(uri)
    }

    public nonisolated func linkOpened(link _: WalletConnectLink) {}

    public func approveSession(proposalId _: String, wallet _: Gemstone.Wallet) async throws {}

    public func rejectSession(proposalId _: String, reason _: GemWalletConnectRejectionReason) async throws {}

    public func answerRequest(topic _: String, requestId _: String) async throws -> GemWalletConnectFailure? {
        nil
    }

    public func rejectRequest(topic _: String, requestId _: String) async throws {}

    public func disconnect(topic _: String) async throws {}

    public nonisolated func authenticationMessage(authenticationId _: String, wallet _: Gemstone.Wallet) throws -> GemWalletConnectAuthMessage {
        .mock()
    }

    public func approveAuthentication(authenticationId _: String, wallet _: Gemstone.Wallet, signature _: String) async throws {}

    public func rejectAuthentication(authenticationId _: String) async throws {}
}
