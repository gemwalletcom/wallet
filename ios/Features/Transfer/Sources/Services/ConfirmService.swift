// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import Blockchain
import ChainService
import EventPresenterService
import ExplorerService
import GemstonePrimitives
import Keystore
import Primitives

public struct ConfirmService: Sendable {
    private let metadataProvider: any TransferMetadataProvidable
    private let inputProvider: ConfirmTransferInputProvider
    private let simulationService: ConfirmSimulationService
    private let transferExecutor: any TransferExecutable
    private let activityService: ActivityService
    private let eventPresenterService: EventPresenterService
    private let keystore: any Keystore
    private let chainService: any ChainServiceable
    private let explorerService: any ExplorerLinkFetchable
    private let addressNameService: AddressNameService

    public init(
        metadataProvider: any TransferMetadataProvidable,
        inputProvider: ConfirmTransferInputProvider,
        simulationService: ConfirmSimulationService,
        transferExecutor: any TransferExecutable,
        activityService: ActivityService,
        eventPresenterService: EventPresenterService,
        keystore: any Keystore,
        chainService: any ChainServiceable,
        explorerService: any ExplorerLinkFetchable,
        addressNameService: AddressNameService,
    ) {
        self.metadataProvider = metadataProvider
        self.inputProvider = inputProvider
        self.simulationService = simulationService
        self.transferExecutor = transferExecutor
        self.activityService = activityService
        self.eventPresenterService = eventPresenterService
        self.keystore = keystore
        self.chainService = chainService
        self.explorerService = explorerService
        self.addressNameService = addressNameService
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
        let input = TransferConfirmationInput(
            data: request.data,
            wallet: request.wallet,
            transactionData: transactionData,
            amount: amount,
            simulation: simulation,
            delegate: request.delegate,
        )
        try await transferExecutor.execute(input: input)
        await eventPresenterService.present(.transfer(request.data))
        if let recent = request.data.type.recentActivityData {
            updateRecent(data: recent, walletId: request.wallet.id)
        }
    }

    public func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        explorerService.addressUrl(chain: chain, address: address)
    }

    public func addressName(chain: Chain, address: String) throws -> AddressName? {
        try addressNameService.getAddressName(chain: chain, address: address)
    }

    public func passwordAuthentication() throws -> KeystoreAuthentication {
        try keystore.getPasswordAuthentication()
    }

    public func defaultPriority(for type: TransferDataType) -> FeePriority {
        chainService.defaultPriority(for: type)
    }
}

// MARK: - Private

private extension ConfirmService {
    func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try activityService.updateRecent(data: data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }
}
