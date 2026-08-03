// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import SigningRequestService
import SwiftUI
import WalletConnectorService

public final class WalletConnectorManager {
    public let presenter: WalletConnectorPresenter

    public init(presenter: WalletConnectorPresenter) {
        self.presenter = presenter
    }
}

// MARK: - WalletConnectorInteractable

extension WalletConnectorManager: WalletConnectorInteractable {
    public func sessionReject(error: any Error) async {
        if let error = error as? SigningRequestError, case .userCancelled = error {
            return
        }
        await MainActor.run { [weak self] in
            guard let self else { return }
            presenter.isPresentingError = error.localizedDescription
        }
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        let value = try await presentSheet(payload: payload, sheetType: { .connectionProposal($0) })
        return try WalletId.from(id: value)
    }

    public func signMessage(payload: SignMessagePayload) async throws -> String {
        try await presentSheet(payload: payload, sheetType: { .signMessage($0) })
    }

    public func sendTransaction(transferData: SigningTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }

    public func signTransaction(transferData: SigningTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }

    public func sendRawTransaction(transferData _: SigningTransferData) async throws -> String {
        throw AnyError.notImplemented
    }

    // MARK: - Private

    private func presentSheet<T: Identifiable & Sendable>(
        payload: T,
        sheetType: @Sendable @escaping (SigningRequestCallback<T>) -> WalletConnectorSheetType,
    ) async throws -> String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)

        let callback = SigningRequestCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }

        await presenter.present(sheet: sheetType(callback))

        do {
            for try await value in stream {
                await presenter.dismissPresentedSheet()
                return value
            }
        } catch {
            await presenter.dismissPresentedSheet()
            throw error
        }
        await presenter.dismissPresentedSheet()
        throw SigningRequestError.userCancelled
    }
}
