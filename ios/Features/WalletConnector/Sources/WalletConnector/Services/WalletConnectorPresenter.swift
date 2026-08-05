// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import PrimitivesComponents

@Observable
public final class WalletConnectorPresenter: SheetPresenting, SigningRequestInteractable, Sendable {
    public let sheets = SheetPresenter<WalletConnectorSheetType>()

    @MainActor
    public var isPresentingError: String?
    @MainActor
    public var isPresentingConnectionBar: Bool = false

    public init() {}

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await present(payload: payload) { .signMessage($0) }
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData) { .transferData($0) }
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await present(payload: transferData) { .transferData($0) }
    }
}
