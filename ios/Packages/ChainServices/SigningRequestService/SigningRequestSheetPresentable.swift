// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives

public protocol SigningRequestSheetPresentable: SigningRequestInteractable {
    associatedtype Sheet: SigningRequestRejectable & Identifiable where Sheet.ID == String

    var sheets: SheetPresenter<Sheet> { get }

    static func signMessageSheet(_ callback: SigningRequestCallback<SignMessagePayload>) -> Sheet
    static func transferSheet(_ callback: SigningRequestCallback<SigningTransferData>) -> Sheet
}

public extension SigningRequestSheetPresentable {
    @MainActor
    var isPresentingSheet: Sheet? {
        get { sheets.isPresentingSheet }
        nonmutating set { sheets.isPresentingSheet = newValue }
    }

    @MainActor
    func complete(type: Sheet) {
        sheets.complete(type: type)
    }

    @MainActor
    func cancelSheet(type: Sheet) {
        sheets.cancelSheet(type: type)
    }

    @MainActor
    func onSheetDismiss() {
        sheets.onSheetDismiss()
    }

    func signMessage(payload: SignMessagePayload) async throws -> String {
        try await sheets.present(payload: payload, sheet: { Self.signMessageSheet($0) })
    }

    func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { Self.transferSheet($0) })
    }

    func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await sheets.present(payload: transferData, sheet: { Self.transferSheet($0) })
    }
}
