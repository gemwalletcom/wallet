// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import PaymentService
import Primitives
import SigningRequestService

public struct PaymentActionResults: Sendable {
    public let results: [String]
    public let transactionHash: String?
}

public struct PaymentActionExecutor: Sendable {
    private let interactor: any SigningRequestInteractable
    private let simulator: any SigningSimulatable
    private let assetsProvider: any PaymentAssetsProvidable

    public init(
        interactor: any SigningRequestInteractable,
        simulator: any SigningSimulatable,
        assetsProvider: any PaymentAssetsProvidable,
    ) {
        self.interactor = interactor
        self.simulator = simulator
        self.assetsProvider = assetsProvider
    }

    @MainActor
    public func perform(
        actions: [PaymentAction],
        paymentId: String,
        appMetadata: TransactionAppMetadata,
        payment: PaymentData,
        wallet: Wallet,
        onSubmitted: @MainActor () -> Void = {},
    ) async throws -> PaymentActionResults {
        var results = [String](repeating: "", count: actions.count)
        var transactionHash: String?
        for (index, action) in actions.enumerated() {
            let value = try await perform(
                action: action,
                id: "\(paymentId).\(index)",
                appMetadata: appMetadata,
                payment: payment,
                wallet: wallet,
            )
            results[index] = value

            if case .sendTransaction = action {
                transactionHash = value
            }
        }
        onSubmitted()
        return PaymentActionResults(results: results, transactionHash: transactionHash)
    }
}

// MARK: - Private

extension PaymentActionExecutor {
    @MainActor
    private func perform(action: PaymentAction, id: String, appMetadata: TransactionAppMetadata, payment: PaymentData, wallet: Wallet) async throws -> String {
        switch action {
        case let .signMessage(chain, message):
            let payload = try await SignMessagePayload(
                id: id,
                chain: chain,
                appMetadata: appMetadata,
                wallet: wallet,
                message: message,
                simulation: simulator.simulateSignMessage(message: message, sessionDomain: appMetadata.url ?? .empty),
                payment: payment,
                expiresAt: payment.expiresAt,
            )
            return try await interactor.signMessage(payload: payload)
        case let .signTransaction(chain, transaction):
            let transferData = try SigningTransferDataFactory.transferData(
                chain: chain,
                appMetadata: appMetadata,
                transaction: transaction,
                outputAction: .sign,
                payment: payment,
            )
            return try await interactor.signTransaction(transferData: SigningTransferData(transferData: transferData, wallet: wallet, simulation: .empty))
        case let .approveToken(_, approval):
            let assetId = payment.quote.amount.assetId
            guard let asset = assetsProvider.assetsData(walletId: wallet.id, assetIds: [assetId]).first?.asset else {
                throw PaymentLinkError.approvalNotBroadcast
            }
            let transferData = TransferData(
                type: .tokenApprove(asset, approval),
                recipientData: RecipientData(
                    recipient: Recipient(name: .none, address: approval.spender, memo: .none),
                    amount: .none,
                ),
                amount: .exact(.zero),
            )
            return try await interactor.sendTransaction(
                transferData: SigningTransferData(transferData: transferData, wallet: wallet, simulation: .empty),
            )
        case let .sendTransaction(chain, transaction):
            let transferData = try SigningTransferDataFactory.transferData(
                chain: chain,
                appMetadata: appMetadata,
                transaction: transaction,
                outputAction: .send,
                payment: payment,
            )
            return try await interactor.sendTransaction(transferData: SigningTransferData(transferData: transferData, wallet: wallet, simulation: .empty))
        }
    }
}
