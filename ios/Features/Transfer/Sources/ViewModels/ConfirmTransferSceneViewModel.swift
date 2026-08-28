// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmData
import func Gemstone.acquireAssetFlow
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
    private let confirmService: ConfirmService
    private let currency: Currency
    private let onComplete: VoidAction

    public init(
        request: ConfirmTransferRequest,
        confirmService: ConfirmService,
        onComplete: VoidAction,
    ) {
        self.request = request
        self.confirmService = confirmService
        self.onComplete = onComplete

        let currency = confirmService.currency
        self.currency = currency
        feeSelection = .preset(confirmService.defaultPriority(for: request.data.type))
        feeAssetSelection = .automatic

        let recipientAddress = request.data.recipient.address
        recipientAddressNameQuery = ObservableQuery(
            AddressNameRequest(chain: request.data.chain, address: recipientAddress),
            initialValue: try? confirmService.addressName(chain: request.data.chain, address: recipientAddress),
        )

        state = ConfirmTransferState(
            simulation: confirmService.simulationState(request: request),
            metadata: try? confirmService.metadata(request: request),
            feeAsset: request.data.type.feeAsset,
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
            explorerLink: confirmService.explorerLink(chain: dataModel.chain, address: senderAddress),
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
            authentication: try? confirmService.passwordAuthentication(),
            isDisabled: isButtonDisabled,
            onAction: { [weak self] in self?.onSelectConfirm() },
        )
    }

    public var detailsViewModel: ConfirmDetailsViewModel {
        ConfirmDetailsViewModel(type: request.data.type, metadata: state.metadata, currency: currency.rawValue)
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
            feeAssets: state.feeAssets,
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
            ConfirmAppViewModel(type: request.data.type)
        case .sender:
            ConfirmSenderViewModel(wallet: request.wallet)
        case .network:
            ConfirmNetworkViewModel(type: request.data.type)
        case .recipient:
            ConfirmRecipientViewModel(
                model: dataModel,
                addressName: recipientAddressNameQuery.value,
                addressLink: confirmService.explorerLink(chain: dataModel.chain, address: dataModel.recipient.address),
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
            explorerLink: { confirmService.explorerLink(chain: dataModel.chain, address: $0) },
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
            Task { await fetch() }
            return
        }
        confirm(confirmData: input.confirmData, amount: amount)
    }

    func fetch() async {
        state.transaction = .loading
        do {
            if state.feeRates.isEmpty {
                let data = try await confirmService.load(
                    request: request,
                    selection: feeSelection,
                    feeAssetSelection: feeAssetSelection,
                )
                state = .loaded(data)
            } else {
                let preload = try await confirmService.preload(
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
        switch acquireAssetFlow(chain: asset.chain.rawValue) {
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
                try await confirmService.confirm(
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
