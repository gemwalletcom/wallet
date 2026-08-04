// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import SigningRequestService

@Observable
public final class WalletConnectorPresenter: SigningRequestInteractable, Sendable {
    public let sheets = SheetPresenter<WalletConnectorSheetType>()

    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false

    public init() {}

    @MainActor
    public var isPresentingSheet: WalletConnectorSheetType? {
        get { sheets.isPresentingSheet }
        set { sheets.isPresentingSheet = newValue }
    }

    @MainActor
    public func complete(type: WalletConnectorSheetType) {
        sheets.complete(type: type)
    }

    @MainActor
    public func cancelSheet(type: WalletConnectorSheetType) {
        sheets.cancelSheet(type: type)
    }

    @MainActor
    public func onSheetDismiss() {
        sheets.onSheetDismiss()
    }

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
