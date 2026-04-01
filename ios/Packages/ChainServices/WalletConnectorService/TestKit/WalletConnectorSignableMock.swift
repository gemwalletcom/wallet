// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import struct Gemstone.SignMessage
import Primitives
import WalletConnectorService
import WalletConnectSign

public struct WalletConnectorSignableMock: WalletConnectorSignable {
    public var allChains: [Chain] = []

    public init() {}

    public func addConnection(connection _: WalletConnection) throws {}
    public func updateSessions(sessions _: [WalletConnectionSession]) throws {}
    public func sessionReject(id _: String, error _: any Error) async throws {}
    public func getCurrentWallet() throws -> Wallet { throw AnyError("not implemented") }
    public func getWallet(id _: WalletId) throws -> Wallet { throw AnyError("not implemented") }
    public func getChains(wallet _: Wallet) -> [Chain] { [] }
    public func getAccounts(wallet _: Wallet, chains _: [Chain]) -> [Primitives.Account] { [] }
    public func getWallets(for _: Session.Proposal) throws -> [Wallet] { [] }
    public func getMethods() -> [WalletConnectionMethods] { [] }
    public func getEvents() -> [WalletConnectionEvents] { [] }
    public func sessionApproval(payload _: WCPairingProposal) async throws -> WalletId { throw AnyError("not implemented") }
    public func signMessage(sessionId _: String, chain _: Chain, message _: SignMessage, simulation _: SimulationResult) async throws -> String { "" }
    public func signTransaction(sessionId _: String, chain _: Chain, transaction _: WalletConnectorTransaction, simulation _: SimulationResult) async throws -> String { "" }
    public func sendTransaction(sessionId _: String, chain _: Chain, transaction _: WalletConnectorTransaction, simulation _: SimulationResult) async throws -> String { "" }
    public func sendRawTransaction(sessionId _: String, chain _: Chain, transaction _: String) async throws -> String { "" }
}
