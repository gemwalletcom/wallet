// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol WalletConnectorInteractable: Sendable {
    func sessionReject(error: any Error) async
    func sessionApproval(payload: WCPairingProposal) async throws -> WalletId
    func authenticationApproval(request: WCAuthenticationRequest) async throws
    func requestExpired(requestId: String) async
    func returnToDapp(name: String, redirect: String?) async
}
