// Copyright (c). Gem Wallet. All rights reserved.

import class Gemstone.GemFeeService
import Store
import GemstoneServices
import struct Gemstone.GemConfirmData
import enum Gemstone.GemConfirmError
import protocol Gemstone.GemConfirmServiceProtocol
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemExplorerServiceProtocol
import protocol Gemstone.GemNameServiceProtocol
import protocol Gemstone.GemPerpetualServiceProtocol
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSendInput
import protocol Gemstone.GemTransactionSigner
import protocol Gemstone.GemTransactionStateServiceProtocol
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
    private let transactionStateService: any GemTransactionStateServiceProtocol
    private let recentAssetsService: any RecentAssetsServiceable
    private let toastPresenter: ToastPresenter
    private let keystore: any Keystore
    private let explorerService: any GemExplorerServiceProtocol
    private let nameService: any GemNameServiceProtocol

    private let feeService: GemFeeService

    public let perpetualService: any GemPerpetualServiceProtocol

    public init(
        metadataProvider: any TransferMetadataProvidable,
        inputProvider: ConfirmTransferInputProvider,
        simulationService: ConfirmSimulationService,
        gemConfirmService: any GemConfirmServiceProtocol,
        signer: any GemTransactionSigner,
        preferencesService: any GemPreferencesServiceProtocol,
        transactionStateService: any GemTransactionStateServiceProtocol,
        recentAssetsService: any RecentAssetsServiceable,
        toastPresenter: ToastPresenter,
        keystore: any Keystore,
        explorerService: any GemExplorerServiceProtocol,
        nameService: any GemNameServiceProtocol,
        feeService: GemFeeService,
        perpetualService: any GemPerpetualServiceProtocol,
    ) {
        self.perpetualService = perpetualService
        self.metadataProvider = metadataProvider
        self.inputProvider = inputProvider
        self.simulationService = simulationService
        self.gemConfirmService = gemConfirmService
        self.signer = signer
        self.preferencesService = preferencesService
        self.transactionStateService = transactionStateService
        self.recentAssetsService = recentAssetsService
        self.toastPresenter = toastPresenter
        self.keystore = keystore
        self.explorerService = explorerService
        self.nameService = nameService
        self.feeService = feeService
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

    func confirm(request: ConfirmTransferRequest, confirmData: GemConfirmData, amount: TransferAmount, simulation: SimulationResult?) async throws {
        let input = GemSendInput(
            wallet: request.wallet.json(),
            confirm: confirmData,
            value: amount.value.description,
            networkFee: amount.networkFee.description,
            simulation: simulation?.json(),
        )
        let result: GemExecuteResult
        do {
            result = try await gemConfirmService.execute(input: input, signer: signer)
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            trackPending()
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, transactions):
            hashes.forEach { request.delegate?(.success($0)) }
            track(wallet: request.wallet, transactions: try transactions.map { try Transaction($0) })
        }
        await toastPresenter.present(.transfer(for: request.data.type))
        if let recent = request.data.type.recentActivityData {
            updateRecent(data: recent, walletId: request.wallet.id)
        }
    }

    private func trackPending() {
        Task {
            do {
                try await transactionStateService.trackPending()
            } catch {
                debugLog("confirm: pending tracking failed \(error)")
            }
        }
    }

    private func track(wallet: Wallet, transactions: [Transaction]) {
        Task {
            do {
                try await transactionStateService.track(walletId: wallet.id.id, transactions: transactions.map { $0.json() })
            } catch {
                debugLog("confirm: transaction tracking failed \(error)")
            }
        }
    }

    public func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        BlockExplorerLink(explorerService.getAddressUrl(chain: chain.rawValue, address: address))
    }

    public func addressName(chain: Chain, address: String) throws -> AddressName? {
        try nameService.addressName(chain: chain.rawValue, address: address).map { try AddressName($0) }
    }

    public var currency: Currency {
        preferencesService.currencyValue
    }

    public func passwordAuthentication() throws -> KeystoreAuthentication {
        try keystore.getPasswordAuthentication()
    }

    public func defaultPriority(for type: TransferDataType) -> FeePriority {
        (try? type.map()).map { feeService.defaultPriority(inputType: $0).map() } ?? .normal
    }
}

// MARK: - Private

private extension ConfirmService {
    func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try recentAssetsService.add(data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }
}
