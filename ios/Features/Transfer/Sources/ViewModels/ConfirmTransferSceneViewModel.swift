// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemConfirmButton
import enum Gemstone.GemConfirmFeeRow
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemFeeRateRows
import Components
import Foundation
import enum Gemstone.GemConfirmError
import struct Gemstone.GemConfirmLoadOptions
import func Gemstone.walletRow
import protocol Gemstone.GemConfirmationProtocol
import struct Gemstone.GemConfirmSimulationState
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
import WalletConnector
import struct Gemstone.SimulationPayloadField
import struct Gemstone.SimulationResult
import struct Gemstone.GemSimulationWarningRow

@Observable
@MainActor
public final class ConfirmTransferSceneViewModel {
    var feeSelection: GemConfirmFeeSelection {
        didSet { feeRates = state.feeRateRows(selection: feeSelection) }
    }
    var feeAssetSelection: FeeAssetSelection
    var assetSelection: AssetId?
    var state: ConfirmTransferState {
        didSet { onStateChange(state: state) }
    }
    private(set) var button: GemConfirmButton
    private(set) var feeRow: GemConfirmFeeRow
    private(set) var feeRates: GemFeeRateRows?

    public var isPresentingSheet: ConfirmTransferSheetType?

    public var isPresentingAlertMessage: AlertMessage?

    private let request: ConfirmTransferRequest

    private let wallet: Wallet
    private let onComplete: ((GemExecuteResult) -> Void)?

    private let confirmation: any GemConfirmationProtocol

    public init(
        request: ConfirmTransferRequest,
        wallet: Wallet,
        confirmation: any GemConfirmationProtocol,
        onComplete: ((GemExecuteResult) -> Void)?,
    ) {
        self.request = request
        self.wallet = wallet
        self.confirmation = confirmation
        self.onComplete = onComplete

        let feeSelection = GemConfirmFeeSelection.priority(priority: request.data.defaultFeePriority())
        let state = ConfirmTransferState(
            transfer: request.data,
            simulation: ConfirmSimulationState(result: request.simulation, chain: request.data.chain),
            screen: confirmation.screen(),
        )
        let screen = state.screen
        self.feeSelection = feeSelection
        self.state = state
        feeAssetSelection = .automatic
        button = screen.button()
        feeRow = screen.feeRow()
        feeRates = state.feeRateRows(selection: feeSelection)
    }

    var selection: ConfirmSelection {
        ConfirmSelection(fee: feeSelection, feeAsset: feeAssetSelection, asset: assetSelection)
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
        state.screen.phase == .confirming
    }

    var isHeaderVisible: Bool {
        guard case .payment = transfer.inputType, transfer.value.isZero else {
            return true
        }
        return state.preload != nil
    }

    var simulationWarnings: [GemSimulationWarningRow] {
        state.simulation.warnings
    }

    var simulationWarningModels: [SimulationWarningViewModel] {
        simulationWarnings.map(SimulationWarningViewModel.init)
    }

    public var payloadModel: SimulationPayloadModel { state.simulation.payload }

    public var transfer: GemTransferData { state.transfer }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            button: button,
            authentication: confirmation.authentication(),
            onAction: { [weak self] in self?.onSelectConfirm() },
        )
    }

    public var detailsViewModel: ConfirmDetailsViewModel {
        ConfirmDetailsViewModel(
            type: transfer.inputType,
            metadata: state.metadata,
            confirmation: confirmation,
        )
    }

    var balanceChangeModels: [ConfirmBalanceChangeViewModel] {
        state.simulation.balanceChanges.map(ConfirmBalanceChangeViewModel.init)
    }

    public var feeModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: state.feeAsset,
            currency: confirmation.currency,
            selection: feeSelection,
            feeRates: feeRates,
            feeAssetPrice: state.metadata?.feePrice,
            feeAmount: state.fee?.fee,
            additionalFees: state.confirmData?.additionalFees ?? [],
            feeAssets: state.feeAssets.map { $0.feeAssetItem(currency: confirmation.currency) },
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
            ListSection(type: .fee, [state.verification == nil ? .networkFee : .verification]),
            ListSection(type: .error, [.error]),
        ].compactMap(\.self)
    }

    private var detailItems: [ConfirmTransferItem] {
        transfer.confirmRows().map { row in
            switch row {
            case .app: .app
            case .sender: .sender
            case .recipient: .recipient
            case .network: .network
            case .memo: .memo
            case .details: .details
            case .paymentAsset: .paymentAsset
            }
        }
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(state: state, currency: confirmation.currency)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarningModels)
        case .app:
            ConfirmAppViewModel(transfer: transfer)
        case .sender:
            ConfirmSenderViewModel(row: walletRow(wallet: wallet.toGem()))
        case .network:
            ConfirmNetworkViewModel(transfer: transfer)
        case .paymentAsset:
            ConfirmPaymentAssetViewModel(transfer: transfer)
        case .recipient:
            ConfirmRecipientViewModel(
                destination: transfer.destination()?.withAddressName(addressName: state.addressName?.toGem()),
                chain: dataModel.chain,
                memo: dataModel.recipient.memo,
                addressName: state.addressName,
                addressLink: explorerLink(chain: dataModel.chain, address: dataModel.recipient.address),
            )
        case .memo:
            ConfirmMemoViewModel(transfer: transfer)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(fieldModels(for: payloadModel.primaryFields))
        case let .balanceChange(index):
            ConfirmTransferItemModel.balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                feeRow: feeRow,
                feeModel: feeModel,
                infoAction: onSelectNetworkFeeInfo,
            )
        case .verification:
            ConfirmVerificationViewModel(infoAction: onSelectVerificationInfo)
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
            prices: state.metadata?.assetPrices ?? [:],
            currency: confirmation.currency.rawValue,
            acquireFlow: { confirmation.acquireAssetFlow(chain: $0.chain.rawValue) },
            networkFeeBuyAmount: Int(confirmation.insufficientNetworkFeeBuyAmount()),
            onGetAsset: { [weak self] asset, buyAmount in self?.onSelectGetAsset(asset, buyAmount: buyAmount) },
        ) else { return }
        isPresentingSheet = .info(sheet)
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(.networkFee(state.feeAsset))
    }

    public func fieldModels(for fields: [SimulationPayloadField]) -> [SimulationPayloadFieldViewModel] {
        payloadModel.fieldModels(
            for: fields,
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

    func onSelectPaymentAsset(_ selection: SelectAssetType) {
        guard state.screen.phase != .loading else { return }
        isPresentingSheet = .paymentAsset(selection)
    }

    public func selectPaymentAsset(_ asset: Asset) {
        isPresentingSheet = nil
        guard asset.id != transfer.asset.id else { return }
        assetSelection = asset.id
    }

    func onSelectVerification() {
        guard let verification = state.verification, let url = URL(string: verification.url) else { return }
        isPresentingSheet = .paymentVerification(url)
    }

    func onSelectVerificationInfo() {
        isPresentingSheet = .info(.paymentVerification)
    }

    public func onPaymentVerified() {
        isPresentingSheet = nil
        Task { await load() }
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
        switch state.screen.action() {
        case .load: Task { await load() }
        case .execute: confirm()
        case .none: break
        }
    }

    func load() async {
        state.screen = state.screen.onLoadStarted()
        do {
            state = try ConfirmTransferState(await confirmation.state(), screen: state.screen)
            let load = try await confirmation.load(options: options(selection: feeSelection, feeAssetSelection: feeAssetSelection))
            state = try ConfirmTransferState(load, screen: state.screen.onLoaded(load: load))
        } catch {
            guard !Task.isCancelled else { return }
            state.screen = state.screen.onLoadFailed(error: error.confirmError)
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
        switch confirmation.acquireAssetFlow(chain: asset.chain.rawValue) {
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

    private func confirm() {
        state.screen = state.screen.onExecuteStarted()
        Task {
            do {
                let result = try await submit(request: request)
                onComplete?(result)
            } catch GemConfirmError.Cancelled {
                state.screen = state.screen.onExecuteCancelled()
            } catch {
                state.screen = state.screen.onExecuteFailed(error: error.confirmError)
                isPresentingAlertMessage = AlertMessage(title: Localized.Errors.transferError, message: error.localizedDescription)
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
        TransferDataViewModel(data: transfer)
    }
}

// MARK: - Confirm

extension ConfirmTransferSceneViewModel {
    func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        confirmation.explorerLink(chain: chain, address: address)
    }

    private func options(selection: GemConfirmFeeSelection, feeAssetSelection: FeeAssetSelection) -> GemConfirmLoadOptions {
        GemConfirmLoadOptions(
            feeSelection: selection,
            feeAssetId: feeAssetSelection.selectedAssetId?.identifier,
            assetId: assetSelection?.identifier,
        )
    }

    func submit(request: ConfirmTransferRequest) async throws -> GemExecuteResult {
        let result: GemExecuteResult
        do {
            result = try await confirmation.execute()
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, _, _):
            hashes.forEach { request.delegate?(.success($0)) }
        }
        return result
    }
}
