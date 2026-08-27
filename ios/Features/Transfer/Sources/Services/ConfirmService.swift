// Copyright (c). Gem Wallet. All rights reserved.

import func Gemstone.defaultFeePriority
import Store
import GemstoneServices
import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSendInput
import protocol Gemstone.GemTransactionSigner
import GemstonePrimitives
import Primitives
import PrimitivesComponents

public struct ConfirmService: Sendable {
    private let metadataProvider: any TransferMetadataProvidable
    private let inputProvider: ConfirmTransferInputProvider
    private let simulationService: ConfirmSimulationService
    private let gemConfirmService: any GemConfirmServiceProtocol
    private let signer: any GemTransactionSigner
    private let preferencesService: any GemPreferencesServiceProtocol
    private let transactionStateScheduler: TransactionStateScheduler
    private let recentActivityStore: RecentActivityStore
    private let toastPresenter: ToastPresenter
    private let keystore: any Keystore
    private let explorerService: any GemExplorerServiceProtocol
    private let addressStore: AddressStore

    public init(
        metadataProvider: any TransferMetadataProvidable,
        inputProvider: ConfirmTransferInputProvider,
        simulationService: ConfirmSimulationService,
        gemConfirmService: any GemConfirmServiceProtocol,
        signer: any GemTransactionSigner,
        preferencesService: any GemPreferencesServiceProtocol,
        transactionStateScheduler: TransactionStateScheduler,
        recentActivityStore: RecentActivityStore,
        toastPresenter: ToastPresenter,
        keystore: any Keystore,
        explorerService: any GemExplorerServiceProtocol,
        addressStore: AddressStore,
    ) {
        self.metadataProvider = metadataProvider
        self.inputProvider = inputProvider
        self.simulationService = simulationService
        self.gemConfirmService = gemConfirmService
        self.signer = signer
        self.preferencesService = preferencesService
        self.transactionStateScheduler = transactionStateScheduler
        self.recentActivityStore = recentActivityStore
        self.toastPresenter = toastPresenter
        self.keystore = keystore
        self.explorerService = explorerService
        self.addressStore = addressStore
    }

    func simulationState(request: ConfirmTransferRequest) -> ConfirmSimulationState {
        simulationService.makeState(data: request.data, simulation: request.simulation)
    }

    func metadata(request: ConfirmTransferRequest) throws -> TransferDataMetadata {
        try metadataProvider.metadata(wallet: request.wallet, data: request.data)
    }

    func load(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferData {
        async let availableFeeAssets = inputProvider.feeAssets(walletId: request.wallet.id, chain: request.data.chain)
        async let preloadResult = preload(request: request, selection: selection, feeAssetSelection: feeAssetSelection)

        let (preload, feeAssets) = try await (preloadResult, availableFeeAssets)
        return await ConfirmTransferData(
            preload: preload,
            simulation: simulationService.updateState(data: request.data, simulation: request.simulation ?? preload.simulation),
            feeAssets: feeAssets,
        )
    }

    func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
        try await inputProvider.load(
            request: request,
            metadata: metadata(request: request),
            selection: selection,
            feeAssetSelection: feeAssetSelection,
        )
    }

    func confirm(request: ConfirmTransferRequest, transactionData: TransactionData, amount: TransferAmount, simulation: SimulationResult?) async throws {
        let input = try GemSendInput(
            wallet: request.wallet.json(),
            transfer: request.data.gem,
            value: amount.value.description,
            fee: transactionData.fee.map(),
            networkFee: amount.networkFee.description,
            metadata: transactionData.metadata,
            simulation: simulation?.json(),
        )
        let result: GemExecuteResult
        do {
            result = try await gemConfirmService.execute(input: input, signer: signer)
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            transactionStateScheduler.trackPendingTransactions()
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, transactions):
            hashes.forEach { request.delegate?(.success($0)) }
            transactionStateScheduler.track(
                wallet: request.wallet,
                transactions: try transactions.map { try Transaction($0) },
                currency: preferencesService.currencyCode,
            )
        }
        await toastPresenter.present(.transfer(for: request.data.type))
        if let recent = request.data.type.recentActivityData {
            updateRecent(data: recent, walletId: request.wallet.id)
        }
    }

    public func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink(explorerService.getAddressUrl(chain: chain.rawValue, address: address))
    }

    public func addressName(chain: Chain, address: String) throws -> AddressName? {
        try addressStore.getAddressName(chain: chain, address: address)
    }

    public func passwordAuthentication() throws -> KeystoreAuthentication {
        try keystore.getPasswordAuthentication()
    }

    public func defaultPriority(for type: TransferDataType) -> FeePriority {
        (try? type.map()).flatMap { FeePriority(rawValue: defaultFeePriority(inputType: $0)) } ?? .normal
    }
}

// MARK: - Private

private extension ConfirmService {
    func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try recentActivityStore.add(data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }
}
