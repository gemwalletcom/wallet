// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmButton
import enum Gemstone.GemConfirmFeeRow
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeRateRows
import struct Gemstone.GemTransferAmount
import Components
import Foundation
import struct Gemstone.GemConfirmData
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmLoadOptions
import protocol Gemstone.GemConfirmSessionProtocol
import struct Gemstone.GemConfirmSimulationState
import protocol Gemstone.GemConfirmTransferServiceProtocol
import enum Gemstone.GemExecuteResult
import protocol Gemstone.GemPreferencesServiceProtocol
import enum Gemstone.GemTransferAmountResult
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServices
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
    var feeSelection: GemConfirmFeeSelection {
        didSet { feeRates = state.feeRateRows(selection: feeSelection) }
    }
    var feeAssetSelection: FeeAssetSelection
    var state: ConfirmTransferState {
        didSet { onStateChange(state: state) }
    }
    private(set) var button: GemConfirmButton
    private(set) var feeRow: GemConfirmFeeRow
    private(set) var feeRates: GemFeeRateRows?

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


    private let request: ConfirmTransferRequest
    private let wallet: Wallet
    private let onComplete: VoidAction

    private var currency: Currency {
        Currency(core: service.getCurrency())
    }

    private let service: any GemConfirmTransferServiceProtocol
    private let session: any GemConfirmSessionProtocol

    public init(
        request: ConfirmTransferRequest,
        wallet: Wallet,
        service: any GemConfirmTransferServiceProtocol,
        session: any GemConfirmSessionProtocol,
        onComplete: VoidAction,
    ) {
        self.request = request
        self.wallet = wallet
        self.service = service
        self.session = session
        self.onComplete = onComplete

        let feeSelection = GemConfirmFeeSelection.priority(priority: request.data.defaultFeePriority())
        let state = ConfirmTransferState(
            transfer: request.data,
            simulation: ConfirmSimulationState(result: request.simulation, chain: request.data.chain),
        )
        let screen = state.screen
        self.feeSelection = feeSelection
        self.state = state
        feeAssetSelection = .automatic
        button = screen.button()
        feeRow = screen.feeRow()
        feeRates = state.feeRateRows(selection: feeSelection)
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
        guard request.data.applicationMetadata?.source == .payment else {
            return true
        }
        return state.transaction.value != nil
    }

    var simulationWarnings: [SimulationWarning] {
        state.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel { state.simulation.payload }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            button: button,
            authentication: service.authentication(),
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
            feeRates: feeRates,
            feeAssetPrice: state.metadata?.feePrice,
            feeAmount: state.transaction.value?.fee.fee,
            feeAssets: state.feeAssets.map { $0.feeAssetItem(currency: currency) },
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
            ConfirmAppViewModel(transfer: request.data)
        case .sender:
            ConfirmSenderViewModel(wallet: wallet)
        case .network:
            ConfirmNetworkViewModel(transfer: request.data)
        case .recipient:
            ConfirmRecipientViewModel(
                destination: request.data.destination(),
                chain: dataModel.chain,
                memo: dataModel.recipient.memo,
                addressName: state.addressName,
                addressLink: explorerLink(chain: dataModel.chain, address: dataModel.recipient.address),
            )
        case .memo:
            ConfirmMemoViewModel(transfer: request.data)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(payloadModel.primaryFields)
        case let .balanceChange(index):
            ConfirmTransferItemModel.balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                feeRow: feeRow,
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
            feePrice: state.metadata?.feePrice,
            currency: currency.rawValue,
            acquireFlow: { service.acquireAssetFlow(chain: $0.chain.rawValue) },
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

    func onSelectConfirm() {
        guard case let .data(input) = state.transaction, case let .success(amount) = input.transferAmount else {
            Task { await load() }
            return
        }
        confirm(confirmData: input.confirmData, amount: amount)
    }

    func load() async {
        do {
            state = try ConfirmTransferState(await session.state())
            state.transaction = .loading
            state = try ConfirmTransferState(await session.load(options: options(selection: feeSelection, feeAssetSelection: feeAssetSelection)))
        } catch {
            guard !Task.isCancelled else { return }
            state.transaction.setError(error)
            debugLog("confirm load error: \(error)")
        }
    }

    private func onStateChange(state: ConfirmTransferState) {
        let screen = state.screen
        button = screen.button()
        feeRow = screen.feeRow()
        feeRates = state.feeRateRows(selection: feeSelection)
        guard let error = state.transactionError else { return }
        switch error {
        case .confirm:
            onSelectListError(error: error)
        case .other:
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
                wallet: wallet,
                amount: buyAmount,
            )
        }
    }

    private func confirm(confirmData: GemConfirmData, amount: GemTransferAmount) {
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
            } catch GemConfirmError.Cancelled {
                state.confirmation = .idle
            } catch {
                state.confirmation = .failed(error)
                debugLog("confirm transaction error: \(error)")
            }
        }
    }

    private var senderAddress: String {
        state.load?.sender.address ?? ""
    }

    public func assetAddress(_ asset: Asset) -> AssetAddress {
        AssetAddress(asset: asset, address: senderAddress)
    }

    public func swapFromAsset(to asset: Asset) -> Asset {
        dataModel.asset.id == asset.id ? state.feeAsset : dataModel.asset
    }

    public var assetAcquisitionWallet: Wallet {
        wallet
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

    private func options(selection: GemConfirmFeeSelection, feeAssetSelection: FeeAssetSelection) -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(
            feeSelection: selection,
            feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
        )
    }

    func submit(request: ConfirmTransferRequest, confirmData: GemConfirmData, amount: GemTransferAmount, simulation: Primitives.SimulationResult?) async throws {
        let result: GemExecuteResult
        do {
            result = try await service.execute(
                confirm: confirmData,
                value: amount.value,
                networkFee: amount.networkFee,
                simulation: simulation?.json(),
            )
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
