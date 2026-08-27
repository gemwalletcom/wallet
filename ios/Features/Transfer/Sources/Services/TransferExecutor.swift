// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSendInput
import struct Gemstone.GemSendResult
import struct Gemstone.GemSignedTransaction
import Foundation
import GemstonePrimitives
import GemstoneServices
import Primitives

public protocol TransferExecutable: Sendable {
    func execute(input: TransferConfirmationInput) async throws
}

public struct TransferExecutor: TransferExecutable {
    private let signer: any TransactionSigning
    private let confirmService: any GemConfirmServiceProtocol
    private let preferencesService: any GemPreferencesServiceProtocol
    private let transactionStateScheduler: TransactionStateScheduler

    public init(
        signer: any TransactionSigning,
        confirmService: any GemConfirmServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        transactionStateScheduler: TransactionStateScheduler,
    ) {
        self.signer = signer
        self.confirmService = confirmService
        self.preferencesService = preferencesService
        self.transactionStateScheduler = transactionStateScheduler
    }

    public func execute(input: TransferConfirmationInput) async throws {
        let signedTransactions = try await signer.sign(
            transfer: input.data,
            transactionData: input.transactionData,
            amount: input.amount,
            wallet: input.wallet,
        )

        switch input.data.type.outputAction {
        case .sign:
            for signedTransaction in signedTransactions {
                input.delegate?(.success(signedTransaction.data))
            }
        case .send:
            try await send(input: input, transactions: signedTransactions)
        }
    }
}

// MARK: - Private

extension TransferExecutor {
    private func send(input: TransferConfirmationInput, transactions: [GemSignedTransaction]) async throws {
        let sendInput = try GemSendInput(
            wallet: input.wallet.json(),
            transfer: input.data.gem,
            value: input.amount.value.description,
            fee: input.transactionData.fee.map(),
            networkFee: input.amount.networkFee.description,
            metadata: input.transactionData.metadata,
            simulation: input.simulation?.json(),
        )
        let result: GemSendResult
        do {
            result = try await confirmService.send(input: sendInput, transactions: transactions)
        } catch let error as GemConfirmError {
            if case let .Broadcast(hashes, msg) = error {
                hashes.forEach { input.delegate?(.success($0)) }
                transactionStateScheduler.trackPendingTransactions()
                throw AnyError(msg)
            }
            throw error
        }
        result.hashes.forEach { input.delegate?(.success($0)) }
        let tracked = try result.transactions.map { try Transaction($0) }
        transactionStateScheduler.track(wallet: input.wallet, transactions: tracked, currency: preferencesService.currencyCode)
    }
}
