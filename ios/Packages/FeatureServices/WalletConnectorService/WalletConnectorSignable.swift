// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol WalletConnectorSignable: Sendable {
    var allChains: [Primitives.Chain] { get }
    func sessionReject(error: any Error) async
    func getCurrentWallet() throws -> Primitives.Wallet
    func getWallet(id: WalletId) throws -> Primitives.Wallet
    func getWallets() throws -> [Primitives.Wallet]
    func sessionApproval(payload: WCPairingProposal) async throws -> WalletId
}
