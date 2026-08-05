// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
@testable import WalletConnector
import WalletConnectorService

final class WalletConnectorInteractableMock: WalletConnectorInteractable, @unchecked Sendable {
    var transactionHash = "1"
    var signature = "signature"
    var signMessageError: Error?
    private var failedSignOnce = false

    private(set) var signMessagePayloads: [SignMessagePayload] = []

    func sessionReject(error _: any Error) async {}

    func sessionApproval(payload _: WCPairingProposal) async throws -> WalletId {
        throw AnyError("Not supported")
    }

    func signMessage(payload: SignMessagePayload) async throws -> String {
        signMessagePayloads.append(payload)
        if let signMessageError, !failedSignOnce {
            failedSignOnce = true
            throw signMessageError
        }
        return signature
    }

    func signTransaction(transferData _: SigningTransferData) async throws -> String {
        transactionHash
    }

    func sendTransaction(transferData _: SigningTransferData) async throws -> String {
        transactionHash
    }

    func sendRawTransaction(transferData _: SigningTransferData) async throws -> String {
        transactionHash
    }
}
