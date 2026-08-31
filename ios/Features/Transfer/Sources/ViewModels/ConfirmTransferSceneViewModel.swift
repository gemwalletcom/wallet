// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmMetadata
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmLoadOptions
import struct Gemstone.GemSendInput
import enum Gemstone.GemConfirmError
import enum Gemstone.GemTransferAmountResult
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemConfirmSceneServiceProtocol
import protocol Gemstone.GemTransactionSigner
import protocol Gemstone.GemPreferencesServiceProtocol
import GemstoneServices
import BigInt
import Components
import Foundation
import GemstonePrimitives
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Swap
import SwiftUI
import Validators
import WalletConnector

@Observable
@MainActor
public final class ConfirmTransferSceneViewModel {
    var feeSelection: FeeSelection
    var feeAssetSelection: FeeAssetSelection
    var state: ConfirmTransferState {
        didSet { onStateChange(state: state) }
    }

    public var isPresentingSheet: ConfirmTransferSheetType?

    public var isPresentingAlertMessage: AlertMessage? {
        get {
            switch state.confirmation {
            case let .failed(error): AlertMessage(title: Localized.Errors.transferError, message: error.localizedDescription)
            case .idle, .confirming: nil
            }
        }
        set {
            if newValue == nil {
                state.confirmation = .idle
            }
        }
    }

    public let recipientAddressNameQuery: ObservableQuery<AddressNameRequest>

    private let request: ConfirmTransferRequest
    private let currency: Currency
    private let onComplete: VoidAction

    private let service: any GemConfirmSceneServiceProtocol
    private let signer: any GemTransactionSigner
    private let keystore: any Keystore
    private let recentAssetsService: any RecentAssetsServiceable
    private let toastPresenter: ToastPresenter

    public init(
        request: ConfirmTransferRequest,
        service: any GemConfirmSceneServiceProtocol,
        signer: any GemTransactionSigner,
        keystore: any Keystore,
        recentAssetsService: any RecentAssetsServiceable,
        toastPresenter: ToastPresenter,
        preferencesService: any GemPreferencesServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.request = request
        self.service = service
        self.signer = signer
        self.keystore = keystore
        self.recentAssetsService = recentAssetsService
        self.toastPresenter = toastPresenter
        self.onComplete = onComplete

        let currency = preferencesService.currencyValue
        self.currency = currency
        feeSelection = .preset(service.defaultFeePriority(inputType: request.data.type.inputType).map())
        feeAssetSelection = .automatic

        let recipientAddress = request.data.recipient.address
        recipientAddressNameQuery = ObservableQuery(
            AddressNameRequest(chain: request.data.chain, address: recipientAddress),
            initialValue: try? service.addressName(chain: request.data.chain, address: recipientAddress),
        )

        state = ConfirmTransferState(
            simulation: Self.simulationState(request: request, service: service),
            metadata: try? Self.metadata(request: request, service: service),
            feeAsset: service.feeAsset(for: request.data.type),
            transaction: .loading,
        )
    }

    var preloadSelection: ConfirmPreloadSelection {
        ConfirmPreloadSelection(fee: feeSelection, feeAsset: feeAssetSelection)
    }

    var title: String {
        dataModel.title
    }

    var websiteURL: URL? {
        dataModel.websiteURL
    }

    var websiteTitle: String {
        Localized.Settings.website
    }

    var senderExplorerContext: ExplorerContextData {
        ExplorerContextData(
            copyValue: .address(value: senderAddress, chain: dataModel.chain),
            explorerLink: explorerLink(chain: dataModel.chain, address: senderAddress),
        )
    }

    var progressMessage: String {
        Localized.Common.loading
    }

    var isConfirming: Bool {
        state.confirmation.isConfirming
    }

    var isHeaderVisible: Bool {
        guard request.data.type.applicationMetadata?.source == .payment else {
            return true
        }
        return state.transaction.value != nil
    }

    var simulationWarnings: [SimulationWarning] {
        state.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel { state.simulation.payload }

    var isButtonDisabled: Bool {
        simulationWarnings.hasCritical
    }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            state: state.transaction,
            authentication: try? keystore.getPasswordAuthentication(),
            isDisabled: isButtonDisabled,
            onAction: { [weak self] in self?.onSelectConfirm() },
        )
    }

    public var detailsViewModel: ConfirmDetailsViewModel {
        ConfirmDetailsViewModel(
            type: request.data.type,
            metadata: state.metadata,
            currency: currency.rawValue,
            service: service,
        )
    }

    var balanceChangeModels: [ConfirmBalanceChangeViewModel] {
        state.simulation.balanceChanges.map(ConfirmBalanceChangeViewModel.init)
    }

    public var feeModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: state.feeAsset,
            currency: currency,
            selection: feeSelection,
            rates: state.feeRates,
            feeAssetPrice: state.metadata?.feePrice,
            feeAmount: state.transaction.value?.fee.fee,
            feeAssets: state.feeAssets.compactMap { try? $0.feeAssetItem(currency: currency) },
            feeService: service.fee(),
            onSelect: { [weak self] in self?.feeSelection = $0 },
            onSelectFeeAsset: { [weak self] in self?.selectFeeAsset($0) },
        )
    }
}

// MARK: - ListSectionProvideable

extension ConfirmTransferSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<ConfirmTransferItem>] {
        [
            ListSection(type: .header, [.header]),
            ListSection(type: .details, detailItems),
            simulationWarnings.isEmpty ? nil : ListSection(type: .warnings, [.warnings]),
            payloadModel.primaryFields.isEmpty ? nil : ListSection(type: .payload, [.payload]),
            balanceChangeModels.isEmpty ? nil : ListSection(type: .balanceChanges, balanceChangeModels.indices.map(ConfirmTransferItem.balanceChange)),
            ListSection(type: .fee, [.networkFee]),
            ListSection(type: .error, [.error]),
        ].compactMap(\.self)
    }

    private var detailItems: [ConfirmTransferItem] {
        if case .generic = request.data.type {
            return [.app, .sender, .network]
        }
        return [.app, .sender, .recipient, .network, .memo, .details]
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(request: request, state: state, currency: currency)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarnings)
        case .app:
            ConfirmAppViewModel(type: request.data.type, applicationMetadataService: service.applicationMetadata())
        case .sender:
            ConfirmSenderViewModel(wallet: request.wallet)
        case .network:
            ConfirmNetworkViewModel(type: request.data.type)
        case .recipient:
            ConfirmRecipientViewModel(
                model: dataModel,
                addressName: recipientAddressNameQuery.value,
                addressLink: explorerLink(chain: dataModel.chain, address: dataModel.recipient.address),
                transferService: service.transfer(),
                onAddContact: onSelectAddRecipientToContacts,
            )
        case .memo:
            ConfirmMemoViewModel(type: request.data.type, recipient: request.data.recipient)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(payloadModel.primaryFields)
        case let .balanceChange(index):
            ConfirmTransferItemModel.balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                state: state.transaction,
                feeModel: feeModel,
                infoAction: onSelectNetworkFeeInfo,
            )
        case .error:
            ConfirmErrorViewModel(
                error: state.transactionError,
                onSelectListError: onSelectListError,
            )
        }
    }
}

// MARK: - Business Logic

extension ConfirmTransferSceneViewModel {
    func onSelectListError(error: ConfirmTransferError) {
        guard let sheet = ConfirmInfoSheetBuilder.build(
            for: error,
            asset: dataModel.asset,
            feePrice: state.metadata?.feePrice,
            currency: currency.rawValue,
            onGetAsset: { [weak self] asset, buyAmount in self?.onSelectGetAsset(asset, buyAmount: buyAmount) },
        ) else { return }
        isPresentingSheet = .info(sheet)
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(.networkFee(state.feeAsset))
    }

    public func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        payloadModel.contextMenuItems(
            for: field,
            explorerLink: { explorerLink(chain: dataModel.chain, address: $0) },
            onOpenURL: { [weak self] in self?.isPresentingSheet = .url($0) },
        )
    }

    func onSelectPayloadDetails() {
        isPresentingSheet = .payloadDetails
    }

    func onSelectOpenWebsiteURL() {
        if let websiteURL {
            isPresentingSheet = .url(websiteURL)
        }
    }

    func onSelectFeePicker() {
        isPresentingSheet = .networkFeeSelector
    }

    func onSelectSwapDetails() {
        isPresentingSheet = .swapDetails
    }

    func onSelectPerpetualDetails(_ model: PerpetualDetailsViewModel) {
        isPresentingSheet = .perpetualDetails(model)
    }

    func onSelectAddRecipientToContacts(_ action: AddContactType) {
        isPresentingSheet = .addContact(action)
    }

    func onSelectConfirm() {
        guard case let .data(input) = state.transaction, case let .success(amount) = input.transferAmount else {
            Task { await load() }
            return
        }
        confirm(confirmData: input.confirmData, amount: amount)
    }

    func load() async {
        state.transaction = .loading
        do {
            if state.feeRates.isEmpty {
                let data = try await load(
                    request: request,
                    selection: feeSelection,
                    feeAssetSelection: feeAssetSelection,
                )
                state = .loaded(data)
            } else {
                let preload = try await preload(
                    request: request,
                    selection: feeSelection,
                    feeAssetSelection: feeAssetSelection,
                )
                state.update(preload)
            }
        } catch {
            guard !Task.isCancelled else { return }
            state.transaction.setError(error)
            debugLog("preload transaction error: \(error)")
        }
    }

    private func onStateChange(state: ConfirmTransferState) {
        guard let error = state.transactionError else { return }
        switch error {
        case .amount, .scan:
            onSelectListError(error: error)
        case .chain, .other:
            break
        }
    }

    private func selectFeeAsset(_ assetId: AssetId) {
        guard state.feeAsset.id != assetId else { return }
        feeAssetSelection = .selected(assetId)
    }
}

// MARK: - Private

extension ConfirmTransferSceneViewModel {
    private func onSelectGetAsset(_ asset: Asset, buyAmount: Int? = nil) {
        switch service.acquireAssetFlow(chain: asset.chain.rawValue) {
        case .options:
            isPresentingSheet = .getAsset(asset, buyAmount: buyAmount)
        case .fiat:
            isPresentingSheet = .fiatConnect(
                assetAddress: AssetAddress(asset: asset, address: senderAddress),
                wallet: request.wallet,
                amount: buyAmount,
            )
        }
    }

    private func confirm(confirmData: GemConfirmData, amount: TransferAmount) {
        guard !state.confirmation.isConfirming else { return }
        state.confirmation = .confirming
        Task {
            do {
                try await submit(
                    request: request,
                    confirmData: confirmData,
                    amount: amount,
                    simulation: state.simulation.result,
                )
                state.confirmation = .idle
                onComplete?()
            } catch {
                if error.isAuthenticationCancelled {
                    state.confirmation = .idle
                } else {
                    state.confirmation = .failed(error)
                    debugLog("confirm transaction error: \(error)")
                }
            }
        }
    }

    private var senderAddress: String {
        (try? request.wallet.account(for: dataModel.chain).address) ?? ""
    }

    public func assetAddress(_ asset: Asset) -> AssetAddress {
        AssetAddress(asset: asset, address: senderAddress)
    }

    public func swapFromAsset(to asset: Asset) -> Asset {
        dataModel.asset.id == asset.id ? state.feeAsset : dataModel.asset
    }

    public var assetAcquisitionWallet: Wallet {
        request.wallet
    }

    private var dataModel: TransferDataViewModel {
        TransferDataViewModel(data: request.data)
    }
}

// MARK: - Confirm

extension ConfirmTransferSceneViewModel {
    static func metadata(request: ConfirmTransferRequest, service: any GemConfirmSceneServiceProtocol) throws -> GemConfirmMetadata {
        try service.metadata(
            walletId: request.wallet.id.id,
            assetId: request.data.type.asset.id.identifier,
            feeAssetId: service.feeAsset(for: request.data.type).id.identifier,
            extraAssetIds: service.assetIds(for: request.data.type).map(\.identifier),
        )
    }

    static func simulationState(request: ConfirmTransferRequest, service: any GemConfirmSceneServiceProtocol) -> ConfirmSimulationState {
        ConfirmSimulationState(
            data: request.data,
            simulation: request.simulation,
            resolved: try? service.simulation(inputType: request.data.type.inputType, simulation: request.simulation?.json()),
            addressNames: [:],
        )
    }

    func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        service.explorerLink(chain: chain, address: address)
    }

    func load(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferData {
        let feeAssets = try service.feeAssets(walletId: request.wallet.id.id, chain: request.data.chain.rawValue)
        let preload = try await preload(request: request, selection: selection, feeAssetSelection: feeAssetSelection)
        return ConfirmTransferData(
            preload: preload,
            simulation: await updatedSimulationState(data: request.data, simulation: request.simulation ?? preload.simulation),
            feeAssets: feeAssets,
        )
    }

    func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
        let metadata = try Self.metadata(request: request, service: service)
        let account = try request.wallet.account(for: request.data.chain)
        let preload: GemConfirmPreload
        do {
            preload = try await service.preload(
                walletId: request.wallet.id.id,
                input: request.data.confirmInput(from: account),
                options: GemConfirmLoadOptions(
                    feeSelection: selection.map(),
                    feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
                ),
            )
        } catch let error as GemConfirmError {
            throw preloadFailureError(metadata: metadata) ?? error
        }
        let feeAsset = try Asset(preload.feeAsset)
        return try ConfirmTransferPreload(
            metadata: preload.metadata,
            input: ConfirmTransferInput(
                confirmData: preload.confirmData,
                fee: preload.confirmData.fee.map(),
                transferAmount: preload.amount.map(asset: request.data.type.asset, feeAsset: feeAsset),
                feeAsset: feeAsset,
            ),
            feeRates: preload.confirmData.feeRates.map { try $0.map() },
            simulation: preload.confirmData.simulation.map { try Primitives.SimulationResult($0) },
        )
    }

    func preloadFailureError(metadata: GemConfirmMetadata) -> TransferAmountCalculatorError? {
        guard service.isInsufficientNetworkFee(feeAssetId: metadata.feeAssetBalance.assetId, feeAvailable: metadata.feeAvailable) else {
            return nil
        }
        guard let feeAssetId = try? AssetId(id: metadata.feeAssetBalance.assetId) else { return nil }
        return .insufficientNetworkFee(feeAssetId.chain.asset, requirement: nil)
    }

    func updatedSimulationState(data: TransferData, simulation: Primitives.SimulationResult?) async -> ConfirmSimulationState {
        do {
            try await service.syncMissingAssets(for: simulation?.simulationAssetIds ?? [])
        } catch {
            debugLog("simulation asset preload error: \(error)")
        }
        let resolved = try? service.simulation(inputType: data.type.inputType, simulation: simulation?.json())
        let requests = ConfirmSimulationState(data: data, simulation: simulation, resolved: resolved, addressNames: [:]).payload.addressRequests
        let names = (try? await service.addressNames(requests: requests)) ?? [:]
        return ConfirmSimulationState(data: data, simulation: simulation, resolved: resolved, addressNames: names)
    }

    func submit(request: ConfirmTransferRequest, confirmData: GemConfirmData, amount: TransferAmount, simulation: Primitives.SimulationResult?) async throws {
        let input = GemSendInput(
            wallet: request.wallet.json(),
            confirm: confirmData,
            value: amount.value.description,
            networkFee: amount.networkFee.description,
            simulation: simulation?.json(),
        )
        let result: GemExecuteResult
        do {
            result = try await service.execute(input: input, signer: signer)
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
            track(wallet: request.wallet, transactions: try transactions.map { try Primitives.Transaction($0) })
        }
        await toastPresenter.present(.transfer(for: request.data.type))
        if let recent = request.data.type.recentActivityData {
            do {
                try recentAssetsService.add(recent, walletId: request.wallet.id)
            } catch {
                debugLog("Failed to update recent activity: \(error)")
            }
        }
    }

    func trackPending() {
        Task {
            do {
                try await service.trackPending()
            } catch {
                debugLog("confirm: pending tracking failed \(error)")
            }
        }
    }

    func track(wallet: Wallet, transactions: [Primitives.Transaction]) {
        Task {
            do {
                try await service.track(walletId: wallet.id, transactions: transactions)
            } catch {
                debugLog("confirm: transaction tracking failed \(error)")
            }
        }
    }
}

private extension Primitives.SimulationResult {
    var simulationAssetIds: [AssetId] {
        balanceChanges.map(\.assetId) + [header?.assetId].compactMap(\.self)
    }
}

private extension GemTransferAmountResult {
    func map(asset: Primitives.Asset, feeAsset: Primitives.Asset) -> Result<Primitives.TransferAmount, TransferAmountCalculatorError> {
        switch self {
        case let .amount(amount):
            guard let value = try? BigInt.from(string: amount.value), let networkFee = try? BigInt.from(string: amount.networkFee) else {
                return .failure(failure(.insufficientBalance(assetId: asset.id, requirement: BalanceRequirement(required: .zero, available: .zero)), asset, feeAsset))
            }
            return .success(Primitives.TransferAmount(value: value, networkFee: networkFee, useMaxAmount: amount.isMaxAmount))
        case let .error(error):
            let mapped = (try? error.map()) ?? .insufficientBalance(
                assetId: error.assetId ?? asset.id,
                requirement: BalanceRequirement(required: .zero, available: .zero),
            )
            return .failure(failure(mapped, asset, feeAsset))
        }
    }

    private func failure(_ error: TransferAmountError, _ asset: Primitives.Asset, _ feeAsset: Primitives.Asset) -> TransferAmountCalculatorError {
        TransferAmountCalculatorError(error, asset: asset, assetFee: feeAsset)
    }
}
