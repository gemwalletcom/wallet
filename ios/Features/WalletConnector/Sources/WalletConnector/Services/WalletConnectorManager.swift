// Copyright (c). Gem Wallet. All rights reserved.

import Primitives
import PrimitivesComponents
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
        if let error = error as? SheetDismissal, case .cancelled = error {
            return
        }
        await MainActor.run { [weak self] in
            guard let self else { return }
            presenter.isPresentingError = error.localizedDescription
        }
    }

    public func sessionApproval(payload: WCPairingProposal) async throws -> WalletId {
        let value = try await presenter.present(payload: payload) { .connectionProposal($0) }
        return try WalletId.from(id: value)
    }

    public func sendRawTransaction(transferData _: SigningTransferData) async throws -> String {
        throw AnyError.notImplemented
    }
}
