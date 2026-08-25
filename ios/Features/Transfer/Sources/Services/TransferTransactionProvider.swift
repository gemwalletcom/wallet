// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Blockchain
import GemstonePrimitives
import Primitives
import ScanService
import Validators

public protocol TransferTransactionProvidable: Sendable {
    func loadTransferTransactionData(
        wallet: Wallet,
        data: TransferData,
        selection: FeeSelection,
        available: BigInt,
    ) async throws -> TransferTransactionData
}

public struct TransferTransactionProvider: TransferTransactionProvidable {
    private let feeRatesProvider: any FeeRateProviding
    private let chainService: any ChainServiceable
    private let scanService: ScanService

    public init(
        chainService: any ChainServiceable,
        scanService: ScanService,
    ) {
        feeRatesProvider = FeeRateService(service: chainService)
        self.chainService = chainService
        self.scanService = scanService
    }

    public func loadTransferTransactionData(
        wallet: Wallet,
        data: TransferData,
        selection: FeeSelection,
        available: BigInt,
    ) async throws -> TransferTransactionData {
        async let getFeeRates = getFeeRates(type: data.type, selection: selection)
        async let getTransactionMetadata = getTransactionMetadata(wallet: wallet, data: data)
        async let getTransactionScan = getTransactionScan(wallet: wallet, data: data)

        let (rates, metadata) = try await (getFeeRates, getTransactionMetadata)
        async let getTransactionData = getTransactionLoad(
            wallet: wallet,
            data: data,
            available: available,
            rate: rates.selected,
            metadata: metadata,
        )
        let scanResult = try await getTransactionScan

        if let scanResult {
            try ScanTransactionValidator.validate(
                transaction: scanResult,
                asset: data.type.asset,
                memo: data.recipientData.recipient.memo,
            )
        }

        return try await TransferTransactionData(
            allRates: rates.rates,
            transactionData: getTransactionData,
            scanResult: scanResult,
        )
    }
}

// MARK: - Private

extension TransferTransactionProvider {
    private func getTransactionLoad(
        wallet: Wallet,
        data: TransferData,
        available: BigInt,
        rate: FeeRate,
        metadata: TransactionLoadMetadata,
    ) async throws -> TransactionData {
        let input = try TransactionInput(
            type: data.type,
            asset: data.type.asset,
            senderAddress: wallet.account(for: data.chain).address,
            destinationAddress: data.recipientData.recipient.address,
            value: data.value,
            balance: available,
            gasPrice: rate.gasPriceType,
            memo: data.recipientData.recipient.memo,
            metadata: metadata,
        )

        return try await chainService.load(input: input)
    }

    private func getTransactionMetadata(wallet: Wallet, data: TransferData) async throws -> TransactionLoadMetadata {
        try await chainService.preload(
            input: TransactionPreloadInput(
                inputType: data.type,
                senderAddress: wallet.account(for: data.chain).address,
                destinationAddress: data.recipientData.recipient.address,
                references: data.recipientData.recipient.references,
            ),
        )
    }

    private func getTransactionScan(wallet: Wallet, data: TransferData) async throws -> ScanTransaction? {
        try await scanService.getScanTransaction(
            chain: data.chain,
            input: TransactionPreloadInput(
                inputType: data.type,
                senderAddress: wallet.account(for: data.chain).address,
                destinationAddress: data.recipientData.recipient.address,
            ),
        )
    }

    private func getFeeRates(type: TransferDataType, selection: FeeSelection) async throws -> (rates: [FeeRate], selected: FeeRate) {
        let rates = try await feeRatesProvider.rates(for: type)
        let selected = try selectFeeRate(from: rates, selection: selection)

        return (rates, selected)
    }
}

func selectFeeRate(from rates: [FeeRate], selection: FeeSelection) throws -> FeeRate {
    switch selection {
    case let .custom(gasPrice):
        let base = rates.first(where: { $0.priority == .normal }) ?? rates.first
        let baseGasPriceType = base?.gasPriceType ?? .regular(gasPrice: gasPrice)
        let gasPriceType = try GasPriceType.custom(base: baseGasPriceType, gasPrice: gasPrice)
        return FeeRate(priority: base?.priority ?? .normal, gasPriceType: gasPriceType)
    case let .preset(priority):
        guard let selected = rates.first(where: { $0.priority == priority }) ?? rates.first else {
            throw ChainCoreError.feeRateMissed
        }
        return selected
    }
}
