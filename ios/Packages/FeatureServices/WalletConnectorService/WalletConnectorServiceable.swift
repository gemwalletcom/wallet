// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.WalletConnectLink

public protocol WalletConnectorServiceable: Sendable {
    func hasSessions() async throws -> Bool
    func setup() async throws
    func connect() async throws
    func disconnect() async
    func pair(uri: String) async throws
    func linkOpened(_ link: WalletConnectLink) async
    func disconnect(sessionId: String) async throws
    func updateSessions() async throws
}
