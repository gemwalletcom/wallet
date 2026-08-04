// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

extension SheetPresenter: SigningRequestInteractable where Sheet == SigningRequestSheetType {
    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await present(payload: payload, sheet: { .signMessage($0) })
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData, sheet: { .transferData($0) })
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData, sheet: { .transferData($0) })
    }
}
