// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectorService

public actor WalletConnectorServiceMock: WalletConnectorServiceable {
    public var isSetup: Bool = false
    private let storedSessions: Bool

    public init(hasSessions: Bool = false) {
        storedSessions = hasSessions
    }

    public func setup() async {
        isSetup = true
    }

    public func pair(uri _: String) async throws {}
    public func disconnect(sessionId _: String) async throws {}
    public func hasSessions() async throws -> Bool { storedSessions }
    public nonisolated func configure() throws {}
    public nonisolated func updateSessions() {}
}
