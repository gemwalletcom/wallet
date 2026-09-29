// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public protocol WalletConnectorServiceable: Sendable {
    func hasSessions() async throws -> Bool
    func setup() async throws
    func pair(uri: String) async throws
    func disconnect(sessionId: String) async throws
    func updateSessions() async throws
}
