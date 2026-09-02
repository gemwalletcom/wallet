// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import struct Gemstone.GemConfirmPreload
import struct Gemstone.GemConfirmSimulationState
import struct Gemstone.GemConfirmLoadOptions
import struct Gemstone.GemSendInput
import enum Gemstone.GemConfirmError
import enum Gemstone.GemTransferAmountResult
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemConfirmTransferServiceProtocol
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
import struct Gemstone.GemTransferData

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
    private let onComplete: VoidAction

    private var currency: Currency {
        Currency(core: service.currency())
    }

    private let service: any GemConfirmTransferServiceProtocol

    public init(
        request: ConfirmTransferRequest,
        service: any GemConfirmTransferServiceProtocol,
        onComplete: VoidAction,
    ) {
        self.request = request
        self.service = service
        self.onComplete = onComplete

        let sceneState = service.sceneState(
            walletId: request.wallet.id.id,
            inputType: request.data.inputType,
            simulation: request.simulation?.json(),
        )
        feeSelection = .preset(sceneState.feePriority.map())
        feeAssetSelection = .automatic

        let recipientAddress = request.data.recipient.address
        recipientAddressNameQuery = ObservableQuery(
            AddressNameRequest(chain: request.data.chain, address: recipientAddress),
            initialValue: try? service.addressName(chain: request.data.chain, address: recipientAddress),
        )

        state = ConfirmTransferState(
            simulation: ConfirmSimulationState(
                data: request.data,
                simulation: request.simulation,
                state: GemConfirmSimulationState(simulation: sceneState.simulation, addressNames: []),
            ),
            metadata: sceneState.metadata,
            feeAsset: sceneState.feeAsset.map(),
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
        guard request.data.inputType.applicationMetadata?.source == .payment else {
            return true
        }
        return state.transaction.value != nil
    }

    var simulationWarnings: [SimulationWarning] {
        state.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel { state.simulation.payload }

    var isButtonDisabled: Bool {
        state.simulation.hasCriticalWarning
    }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            state: state.transaction,
            authentication: service.authentication(),
            isDisabled: isButtonDisabled,
            onAction: { [weak self] in self?.onSelectConfirm() },
        )
    }

    public var detailsViewModel: ConfirmDetailsViewModel {
        ConfirmDetailsViewModel(
            type: request.data.inputType,
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
        if case .generic = request.data.inputType {
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
            ConfirmAppViewModel(type: request.data.inputType, shortName: request.data.inputType.applicationShortName())
        case .sender:
            ConfirmSenderViewModel(wallet: request.wallet)
        case .network:
            ConfirmNetworkViewModel(type: request.data.inputType)
        case .recipient:
            ConfirmRecipientViewModel(
                model: dataModel,
                addressName: recipientAddressNameQuery.value,
                addressLink: explorerLink(chain: dataModel.chain, address: dataModel.recipient.address),
                outputAction: request.data.inputType.outputAction,
                onAddContact: onSelectAddRecipientToContacts,
            )
        case .memo:
            ConfirmMemoViewModel(type: request.data.inputType, recipient: request.data.recipient)
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
    func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        service.explorerLink(chain: chain, address: address)
    }

    func load(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferData {
        let scene = try await service.loadScene(
            walletId: request.wallet.id.id,
            input: try request.confirmInput(),
            options: options(selection: selection, feeAssetSelection: feeAssetSelection),
            simulation: request.simulation?.json(),
        )
        let preload = try ConfirmTransferPreload(scene.preload)
        return ConfirmTransferData(
            preload: preload,
            simulation: ConfirmSimulationState(
                data: request.data,
                simulation: request.simulation ?? preload.simulation,
                state: scene.simulation,
            ),
            feeAssets: scene.feeAssets,
        )
    }

    func preload(request: ConfirmTransferRequest, selection: FeeSelection, feeAssetSelection: FeeAssetSelection) async throws -> ConfirmTransferPreload {
        try ConfirmTransferPreload(
            await service.preload(
                walletId: request.wallet.id.id,
                input: try request.confirmInput(),
                options: options(selection: selection, feeAssetSelection: feeAssetSelection),
            )
        )
    }

    private func options(selection: FeeSelection, feeAssetSelection: FeeAssetSelection) -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(
            feeSelection: selection.map(),
            feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
        )
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
            result = try await service.execute(input: input)
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, _):
            hashes.forEach { request.delegate?(.success($0)) }
        }
    }
}
