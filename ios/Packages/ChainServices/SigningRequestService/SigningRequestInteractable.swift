// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol SigningRequestInteractable: Sendable {
    func signMessage(payload: SignMessagePayload) async throws -> String
    func signTransaction(transferData: SigningTransferData) async throws -> String
    func sendTransaction(transferData: SigningTransferData) async throws -> String
}
