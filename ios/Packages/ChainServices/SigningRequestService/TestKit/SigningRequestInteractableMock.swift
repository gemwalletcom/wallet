// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService

public final class SigningRequestInteractableMock: SigningRequestInteractable, @unchecked Sendable {
    public init() {}

    public var transactionHash = "1"
    public var signature = "signature"
    public var signMessageError: Error?
    private var failedSignOnce = false

    public private(set) var signMessagePayloads: [SignMessagePayload] = []
    public private(set) var sentTransferData: [SigningTransferData] = []

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        signMessagePayloads.append(payload)
        if let signMessageError, !failedSignOnce {
            failedSignOnce = true
            throw signMessageError
        }
        return signature
    }

    public func signTransaction(transferData _: SigningTransferData) async throws -> String {
        transactionHash
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        sentTransferData.append(transferData)
        return transactionHash
    }
}
