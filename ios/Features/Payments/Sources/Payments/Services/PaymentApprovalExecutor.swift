// Copyright (c). Gem Wallet. All rights reserved.

import AssetsService
import BigInt
import ChainService
import BalanceService
import Foundation
import Keystore
import PaymentService
import Primitives
import ScanService
import Signer
import TransactionStateService
import TransferService
import Blockchain

public struct PaymentApprovalExecutor: PaymentApprovalExecutable {
    private static let confirmationTimeout: Duration = .seconds(120)

    private let keystore: any Keystore
    private let assetsService: AssetsService
    private let chainServiceFactory: any ChainServiceFactorable
    private let scanService: ScanService
    private let assetsEnabler: any AssetsEnabler
    private let transactionStateScheduler: TransactionStateScheduler

    public init(
        keystore: any Keystore,
        assetsService: AssetsService,
        chainServiceFactory: any ChainServiceFactorable,
        scanService: ScanService,
        assetsEnabler: any AssetsEnabler,
        transactionStateScheduler: TransactionStateScheduler,
    ) {
        self.keystore = keystore
        self.assetsService = assetsService
        self.chainServiceFactory = chainServiceFactory
        self.scanService = scanService
        self.assetsEnabler = assetsEnabler
        self.transactionStateScheduler = transactionStateScheduler
    }

    public func getApprovalFee(assetId: AssetId, approval: ApprovalData, wallet: Wallet) async throws -> BigInt {
        try await load(assetId: assetId, approval: approval, wallet: wallet).transaction.transactionData.fee.fee
    }

    public func approve(assetId: AssetId, approval: ApprovalData, wallet: Wallet) async throws -> String {
        let loaded = try await load(assetId: assetId, approval: approval, wallet: wallet)
        return try await broadcast(data: loaded.data, transaction: loaded.transaction, wallet: wallet, chainService: loaded.chainService)
    }

    public func waitForApproval(hash: String, assetId: AssetId, wallet: Wallet) async throws {
        let chain = assetId.chain
        try await TransactionConfirmationWaiter(chainService: chainServiceFactory.service(for: chain))
            .wait(hash: hash, chain: chain, senderAddress: try wallet.account(for: chain).address)
    }
}

private struct LoadedApproval {
    let data: TransferData
    let transaction: TransferTransactionData
    let chainService: any ChainServiceable
}

// MARK: - Private

extension PaymentApprovalExecutor {
    private func load(assetId: AssetId, approval: ApprovalData, wallet: Wallet) async throws -> LoadedApproval {
        let asset = try await assetsService.getOrFetchAsset(for: assetId)
        let chainService = chainServiceFactory.service(for: assetId.chain)
        let account = try wallet.account(for: assetId.chain)
        let data = TransferData(
            type: .tokenApprove(asset, approval),
            recipientData: RecipientData(recipient: Recipient(name: .none, address: approval.spender, memo: .none), amount: .none),
            amount: .exact(.zero),
        )
        let available = try await chainService.coinBalance(for: account.address).balance.available
        let transaction = try await TransferTransactionProvider(chainService: chainService, scanService: scanService)
            .loadTransferTransactionData(wallet: wallet, data: data, selection: .preset(.normal), available: available)

        return LoadedApproval(data: data, transaction: transaction, chainService: chainService)
    }

    private func broadcast(
        data: TransferData,
        transaction: TransferTransactionData,
        wallet: Wallet,
        chainService: any ChainServiceable,
    ) async throws -> String {
        let input = TransferConfirmationInput(
            data: data,
            wallet: wallet,
            transactionData: transaction.transactionData,
            amount: TransferAmount(value: .zero, networkFee: transaction.transactionData.fee.fee, useMaxAmount: false),
            simulation: .none,
            delegate: .none,
        )
        let hashes = try await TransferExecutor(
            signer: TransactionSigner(keystore: keystore),
            chainService: chainService,
            assetsEnabler: assetsEnabler,
            transactionStateScheduler: transactionStateScheduler,
        ).execute(input: input)

        guard let hash = hashes.first else {
            throw PaymentLinkError.approvalNotBroadcast
        }
        return hash
    }
}
