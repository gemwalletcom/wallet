// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.WalletConnectLink
import WalletConnectorService

public actor WalletConnectorServiceMock: WalletConnectorServiceable {
    public var isSetup: Bool = false
    public private(set) var pairedURIs: [String] = []
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

    public func connect() async {}

    public func disconnect() async {}

    public func pair(uri: String) async throws {
        if let pairError {
            throw pairError
        }
        pairedURIs.append(uri)
    }

    public func linkOpened(_: WalletConnectLink) {}

    public func disconnect(sessionId _: String) async throws {
        if let disconnectError {
            throw disconnectError
        }
    }

    public func hasSessions() async throws -> Bool {
        storedSessions
    }

    public func updateSessions() async throws {}
}
