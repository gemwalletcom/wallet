// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletConnectSigner
import struct Gemstone.GemWalletConnectMessageRequest
import struct Gemstone.GemWalletConnectTransactionRequest
import GemstonePrimitives
import Primitives
import SwiftUI
import WalletConnectorService

public final class WalletConnectorInteractor {
    private let presenter: WalletConnectorPresenter

    public init(presenter: WalletConnectorPresenter) {
        self.presenter = presenter
    }
}

// MARK: - WalletConnectorInteractable

extension WalletConnectorInteractor: WalletConnectorInteractable {
    public func sessionReject(error: any Error) async {
        guard !error.isCancelled else { return }
        switch error {
        case ConnectionsError.userCancelled: return
        default: break
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
}

// MARK: - GemWalletConnectSigner

extension WalletConnectorInteractor: GemWalletConnectSigner {
    public func signMessage(request: GemWalletConnectMessageRequest) async throws -> String {
        let payload = try SignMessagePayload(request)
        return try await present { try await presentSheet(payload: payload, sheetType: { .signMessage($0) }) }
    }

    public func signTransaction(request: GemWalletConnectTransactionRequest) async throws -> String {
        let data = try WCTransferData(request)
        return try await present { try await presentSheet(payload: data, sheetType: { .transferData($0) }) }
    }

    private func present(_ action: () async throws -> String) async throws -> String {
        do {
            return try await action()
        } catch ConnectionsError.userCancelled {
            throw GemServiceError.Cancelled
        }
    }
}

// MARK: - Private

extension WalletConnectorInteractor {
    private func presentSheet<T: Identifiable & Sendable>(
        payload: T,
        sheetType: @Sendable @escaping (TransferDataCallback<T>) -> WalletConnectorSheetType,
    ) async throws -> String {
        let (stream, continuation) = AsyncThrowingStream.makeStream(of: String.self)

        let callback = TransferDataCallback(payload: payload) {
            continuation.yield(with: $0)
            continuation.finish()
        }

        await MainActor.run { [weak self] in
            self?.presenter.isPresentingSheet = sheetType(callback)
        }

        for try await value in stream {
            return value
        }
        throw ConnectionsError.userCancelled
    }
}
