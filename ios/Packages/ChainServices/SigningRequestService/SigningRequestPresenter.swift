// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public final class SigningRequestPresenter: SigningRequestInteractable, Sendable {
    public let sheets = SheetPresenter<SigningRequestSheetType>()

    public init() {}

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await sheets.present(payload: payload, sheet: { .signMessage($0) })
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { .transferData($0) })
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { .transferData($0) })
    }
}
