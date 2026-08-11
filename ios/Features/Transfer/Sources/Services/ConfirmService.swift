// Copyright (c). Gem Wallet. All rights reserved.

import ActivityService
import AddressNameService
import Blockchain
import ChainService
import EventPresenterService
import ExplorerService
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

    func load(request: ConfirmTransferRequest, selection: FeeSelection) async throws -> ConfirmTransferData {
        async let simulation = simulationService.updateState(data: request.data, simulation: request.simulation)
        let preload = try await preload(request: request, selection: selection)

        return ConfirmTransferData(preload: preload, simulation: await simulation)
    }

    func preload(request: ConfirmTransferRequest, selection: FeeSelection) async throws -> ConfirmTransferPreload {
        try await inputProvider.load(request: request, metadata: metadata(request: request), selection: selection)
    }

    func confirm(request: ConfirmTransferRequest, transactionData: TransactionData, amount: TransferAmount) async throws {
        let input = TransferConfirmationInput(
            data: request.data,
            wallet: request.wallet,
            transactionData: transactionData,
            amount: amount,
            simulation: request.simulation,
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

extension ConfirmService {
    private func updateRecent(data: RecentActivityData, walletId: WalletId) {
        do {
            try activityService.updateRecent(data: data, walletId: walletId)
        } catch {
            debugLog("Failed to update recent activity: \(error)")
        }
    }
}
