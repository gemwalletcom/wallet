// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemServiceError
import protocol Gemstone.GemWalletConnectSigner
import struct Gemstone.GemWalletConnectSignRequest
import GemstonePrimitives
import Primitives
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
        switch error {
        case ConnectionsError.userCancelled, GemServiceError.Cancelled: return
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

    private func signMessage(payload: SignMessagePayload) async throws -> String {
        try await presentSheet(payload: payload, sheetType: { .signMessage($0) })
    }

    private func sendTransaction(transferData: WCTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }

    private func signTransaction(transferData: WCTransferData) async throws -> String {
        try await presentSheet(payload: transferData, sheetType: { .transferData($0) })
    }
}

// MARK: - GemWalletConnectSigner

extension WalletConnectorManager: GemWalletConnectSigner {
    public func sign(request: GemWalletConnectSignRequest) async throws -> String {
        let chain = try request.chain.map()
        let session = try WalletConnectionSession(request.session)
        let wallet = try Wallet(request.wallet)
        let simulation = try SimulationResult(request.simulation)

        switch request.payload {
        case let .message(message):
            let payload = SignMessagePayload(chain: chain, session: session, wallet: wallet, message: message, simulation: simulation)
            return try await interact { try await signMessage(payload: payload) }
        case let .transaction(transfer, action):
            let data = try WCTransferData(transferData: TransferData(transfer), wallet: wallet, simulation: simulation)
            return try await interact {
                switch action {
                case .sign: try await signTransaction(transferData: data)
                case .send: try await sendTransaction(transferData: data)
                }
            }
        }
    }

    private func interact(_ action: () async throws -> String) async throws -> String {
        do {
            return try await action()
        } catch ConnectionsError.userCancelled {
            throw GemServiceError.Cancelled
        }
    }
}

// MARK: - Private

extension WalletConnectorManager {

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
