// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol WalletConnectorSignable: Sendable {
    var allChains: [Primitives.Chain] { get }

    func addConnection(connection: WalletConnection) throws
    func updateSessions(sessions: [WalletConnectionSession]) throws
    func sessionReject(error: any Error) async
    func sessionReject(id: String, error: any Error) async throws
    func getCurrentWallet() throws -> Primitives.Wallet
    func getWallet(id: WalletId) throws -> Primitives.Wallet
    func getWallets() throws -> [Primitives.Wallet]
    func getMethods() -> [WalletConnectionMethods]
    func getEvents() -> [WalletConnectionEvents]
    func sessionApproval(payload: WCPairingProposal) async throws -> WalletId
}
