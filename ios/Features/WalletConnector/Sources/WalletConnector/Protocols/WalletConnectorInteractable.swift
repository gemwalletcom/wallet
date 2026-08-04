// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService

public protocol WalletConnectorInteractable: Sendable {
    func sessionReject(error: any Error) async
    func sessionApproval(payload: WCPairingProposal) async throws -> WalletId
    func sendRawTransaction(transferData: SigningTransferData) async throws -> String
}
