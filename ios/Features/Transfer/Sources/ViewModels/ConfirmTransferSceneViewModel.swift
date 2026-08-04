// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Blockchain
import Components
import Foundation
import GemstonePrimitives
import InfoSheet
import Localization
import Preferences
import Primitives
import PrimitivesComponents
import Store
import Swap
import SwiftUI
import Validators

@Observable
@MainActor
public final class ConfirmTransferSceneViewModel {
    public var feeModel: NetworkFeeSceneViewModel
    var state: ConfirmTransferState {
        didSet { onStateChange(state: state) }
    }

    public var isPresentingSheet: ConfirmTransferSheetType?

    public let expiryCountdown: ExpiryCountdown

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
        expiryCountdown = ExpiryCountdown(expiresAt: request.data.type.payment?.expiresAt)

        let currency = Currency(rawValue: Preferences.standard.currency) ?? .usd
        self.currency = currency
        feeModel = NetworkFeeSceneViewModel(
            chain: request.data.chain,
            feeAsset: request.data.type.asset.feeAsset,
            priority: confirmService.defaultPriority(for: request.data.type),
            currency: currency,
            mode: .custom,
        )

        let recipientAddress = request.data.recipientData.recipient.address
        recipientAddressNameQuery = ObservableQuery(
            AddressNameRequest(chain: request.data.chain, address: recipientAddress),
            initialValue: try? confirmService.addressName(chain: request.data.chain, address: recipientAddress),
        )

        state = ConfirmTransferState(
            simulation: confirmService.simulationState(request: request),
            metadata: try? confirmService.metadata(request: request),
            transaction: .loading,
        )
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

    var simulationWarnings: [SimulationWarning] {
        state.simulation.warnings
    }

    public var primaryPayloadFields: [SimulationPayloadField] {
        state.simulation.primaryFields
    }

    public var secondaryPayloadFields: [SimulationPayloadField] {
        state.simulation.secondaryFields
    }

    var hasPayloadDetails: Bool {
        state.simulation.hasDetails
    }

    var isButtonDisabled: Bool {
        expiryCountdown.isExpired || simulationWarnings.contains(where: { $0.severity == .critical })
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
        ConfirmDetailsViewModel(type: request.data.type, metadata: state.metadata)
    }

    var balanceChangeModels: [ConfirmBalanceChangeViewModel] {
        state.simulation.balanceChanges.map(ConfirmBalanceChangeViewModel.init)
    }
}

// MARK: - ListSectionProvideable

extension ConfirmTransferSceneViewModel {
    private var paymentExpiryItemModel: ConfirmTransferItemModel {
        guard let expiresAt = request.data.type.payment?.expiresAt else {
            return .empty
        }
        return .paymentExpiry(title: Localized.Transfer.paymentExpiresIn, expiresAt: expiresAt)
    }
}

extension ConfirmTransferSceneViewModel: ListSectionProvideable {
    public var sections: [ListSection<ConfirmTransferItem>] {
        [
            ListSection(type: .header, [.header]),
            ListSection(type: .details, detailItems),
            paymentItems.isEmpty ? nil : ListSection(type: .paymentExpiry, paymentItems),
            simulationWarnings.isEmpty ? nil : ListSection(type: .warnings, [.warnings]),
            primaryPayloadFields.isEmpty ? nil : ListSection(type: .payload, [.payload]),
            balanceChangeModels.isEmpty ? nil : ListSection(type: .balanceChanges, balanceChangeModels.indices.map(ConfirmTransferItem.balanceChange)),
            ListSection(type: .fee, [.networkFee]),
            ListSection(type: .error, [.error]),
        ].compactMap(\.self)
    }

    private var detailItems: [ConfirmTransferItem] {
        if case .generic = request.data.type {
            return [.app, .sender, .network]
        }
        if case .payment = request.data.type {
            return [.app, .sender, .recipient, .network]
        }
        return [.app, .sender, .recipient, .network, .memo, .details]
    }

    private var paymentItems: [ConfirmTransferItem] {
        guard request.data.type.payment != nil else {
            return []
        }
        return [.paymentExpiry]
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(request: request, state: state)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarnings)
        case .paymentExpiry:
            paymentExpiryItemModel
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
            ConfirmMemoViewModel(type: request.data.type, recipientData: request.data.recipientData)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(primaryPayloadFields)
        case let .balanceChange(index):
            ConfirmTransferItemModel.balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                state: state.transaction,
                title: feeModel.title,
                value: feeModel.value,
                fiatValue: feeModel.fiatValue,
                selectable: feeModel.showFeeDetails,
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
        isPresentingSheet = .info(.networkFee(dataModel.chain))
    }

    public func contextMenuItems(for field: SimulationPayloadField) -> [ContextMenuItemType] {
        var items = payloadFieldViewModel(for: field).contextMenuItems
        if field.fieldType == .address {
            let link = confirmService.explorerLink(chain: dataModel.chain, address: field.value)
            items.append(.url(title: Localized.Transaction.viewOn(link.name), onOpen: { [weak self] in
                if let url = URL(string: link.link) {
                    self?.isPresentingSheet = .url(url)
                }
            }))
        }
        return items
    }

    public func payloadFieldViewModel(for field: SimulationPayloadField) -> SimulationPayloadFieldViewModel {
        SimulationPayloadFieldViewModel(
            field: field,
            chain: dataModel.chain,
            addressName: state.simulation.addressName(chain: dataModel.chain, for: field),
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
        confirm(transactionData: input.transactionData, amount: amount)
    }

    func fetch() async {
        state.transaction = .loading
        feeModel.reset()
        do {
            let data = try await confirmService.load(request: request, selection: feeModel.selection)
            state = .loaded(data)
            feeModel.update(rates: data.input.feeRates, feeAssetPrice: data.metadata.feePrice)
            feeModel.update(feeAmount: data.input.transactionData.fee.fee)
        } catch {
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
}

// MARK: - Private

extension ConfirmTransferSceneViewModel {
    private func onSelectGetAsset(_ asset: Asset, buyAmount: Int? = nil) {
        if asset.chain == .tron {
            isPresentingSheet = .getAsset(asset, buyAmount: buyAmount)
        } else {
            isPresentingSheet = .fiatConnect(
                assetAddress: AssetAddress(asset: asset, address: senderAddress),
                wallet: request.wallet,
                amount: buyAmount,
            )
        }
    }

    private func confirm(transactionData: TransactionData, amount: TransferAmount) {
        guard !state.confirmation.isConfirming else { return }
        state.confirmation = .confirming
        Task {
            do {
                try await confirmService.confirm(request: request, transactionData: transactionData, amount: amount)
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
        dataModel.asset.id == asset.id ? dataModel.asset.feeAsset : dataModel.asset
    }

    public var assetAcquisitionWallet: Wallet {
        request.wallet
    }

    private var dataModel: TransferDataViewModel {
        TransferDataViewModel(data: request.data)
    }
}
