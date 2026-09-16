// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import WalletConnectorService

public actor WalletConnectorServiceMock: WalletConnectorServiceable {
    public var isSetup: Bool = false
    private let storedSessions: Bool
    private let pairError: Error?
    private let disconnectError: Error?

    public init(hasSessions: Bool = false, pairError: Error? = nil, disconnectError: Error? = nil) {
        storedSessions = hasSessions
        self.pairError = pairError
        self.disconnectError = disconnectError
    }

    public func setup() async {
        isSetup = true
    }

    public func pair(uri _: String) async throws {
        if let pairError { throw pairError }
    }

    public func disconnect(sessionId _: String) async throws {
        if let disconnectError { throw disconnectError }
    }
    public func hasSessions() async throws -> Bool { storedSessions }
    public nonisolated func configure() throws {}
    public nonisolated func updateSessions() {}
}
