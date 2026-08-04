// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Foundation
import PaymentService
import Primitives
import SigningRequestService

public struct PaymentActionExecutor: Sendable {
    private let interactor: any SigningRequestInteractable
    private let approvalExecutor: any PaymentApprovalExecutable
    private let simulator: any SigningSimulatable
    private let assetsProvider: any PaymentAssetsProvidable

    public init(
        interactor: any SigningRequestInteractable,
        simulator: any SigningSimulatable,
        approvalExecutor: any PaymentApprovalExecutable,
        assetsProvider: any PaymentAssetsProvidable,
    ) {
        self.interactor = interactor
        self.approvalExecutor = approvalExecutor
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
        let networkFee = try await getApprovalsFee(actions: actions, payment: payment, wallet: wallet)

        var signatures: [(index: Int, action: PaymentAction)] = []
        var spends: [(index: Int, action: PaymentAction)] = []
        for (index, action) in actions.enumerated() {
            switch action {
            case .signMessage, .signTransaction:
                signatures.append((index, action))
            case .approveToken, .sendTransaction:
                spends.append((index, action))
            }
        }

        var results = [String](repeating: "", count: actions.count)
        var transactionHash: String?
        var submitted = false
        for (index, action) in signatures + spends {
            let value = try await perform(
                action: action,
                id: "\(paymentId).\(index)",
                appMetadata: appMetadata,
                payment: payment,
                networkFee: networkFee,
                wallet: wallet,
            )
            results[index] = value

            switch action {
            case .sendTransaction:
                transactionHash = value
            case .approveToken:
                if !submitted {
                    submitted = true
                    onSubmitted()
                }
                try await approvalExecutor.waitForApproval(hash: value, assetId: payment.quote.amount.assetId, wallet: wallet)
            case .signMessage, .signTransaction:
                break
            }
        }
        if !submitted {
            onSubmitted()
        }
        return PaymentActionResults(results: results, transactionHash: transactionHash)
    }
}

public struct PaymentActionResults: Sendable {
    public let results: [String]
    public let transactionHash: String?
}

// MARK: - Private

extension PaymentActionExecutor {
    @MainActor
    private func perform(action: PaymentAction, id: String, appMetadata: TransactionAppMetadata, payment: PaymentData, networkFee: AssetValuePrice?, wallet: Wallet) async throws -> String {
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
                networkFee: networkFee,
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
            return try await approvalExecutor.approve(assetId: payment.quote.amount.assetId, approval: approval, wallet: wallet)
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

// MARK: - Private

extension PaymentActionExecutor {
    private func getApprovalsFee(actions: [PaymentAction], payment: PaymentData, wallet: Wallet) async throws -> AssetValuePrice {
        let approvals = actions.compactMap { action -> (chain: Chain, approval: ApprovalData)? in
            guard case let .approveToken(chain, approval) = action else {
                return .none
            }
            return (chain, approval)
        }
        let chain = approvals.first?.chain ?? payment.quote.amount.assetId.chain
        var value = BigInt.zero
        for approval in approvals {
            value += try await approvalExecutor.getApprovalFee(assetId: payment.quote.amount.assetId, approval: approval.approval, wallet: wallet)
        }
        return AssetValuePrice(
            asset: chain.asset,
            value: value,
            price: assetsProvider.assetsData(walletId: wallet.id, assetIds: [chain.assetId]).first?.price,
        )
    }
}
