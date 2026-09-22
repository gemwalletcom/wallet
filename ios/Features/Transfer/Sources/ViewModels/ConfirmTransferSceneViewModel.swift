// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import protocol Gemstone.GemConfirmationProtocol
import struct Gemstone.GemConfirmButton
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmFeeRow
import enum Gemstone.GemConfirmFeeSelection
import enum Gemstone.GemConfirmRowContent
import struct Gemstone.GemConfirmSimulationState
import struct Gemstone.GemFeeRateRows
import enum Gemstone.GemListRow
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSimulationPayloadRow
import enum Gemstone.GemSubmitResult
import struct Gemstone.GemSwapPairSelection
import enum Gemstone.GemTransferAmountResult
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult
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
    private(set) var rowContents: [GemConfirmRowContent]

    public var isPresentingSheet: ConfirmTransferSheetType?

    public var isPresentingAlertMessage: AlertMessage?

    private let request: ConfirmTransferRequest
    private let wallet: Wallet
    private let onComplete: ((GemSubmitResult) -> Void)?

    private let confirmation: any GemConfirmationProtocol

    public init(
        request: ConfirmTransferRequest,
        wallet: Wallet,
        confirmation: any GemConfirmationProtocol,
        onComplete: ((GemSubmitResult) -> Void)?,
    ) {
        self.request = request
        self.wallet = wallet
        self.confirmation = confirmation
        self.onComplete = onComplete

        let feeSelection = GemConfirmFeeSelection.priority(priority: request.data.defaultFeePriority())
        let state = ConfirmTransferState(
            transfer: request.data,
            simulation: ConfirmSimulationState(result: request.simulation),
            screen: confirmation.screen(),
        )
        let screen = state.screen
        self.feeSelection = feeSelection
        self.state = state
        feeAssetSelection = .automatic
        button = screen.button()
        feeRow = screen.feeRow()
        feeRates = state.feeRateRows(selection: feeSelection)
        rowContents = confirmation.rowContents(addressName: state.addressName?.toGem())
    }

    var preloadSelection: ConfirmPreloadSelection {
        ConfirmPreloadSelection(fee: feeSelection, feeAsset: feeAssetSelection, asset: assetSelection)
    }

    var payloadDetailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    var title: String {
        transfer.title().title
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

    var simulationWarnings: [GemListRow] {
        state.simulation.warnings
    }

    public var payloadModel: SimulationPayloadModel { state.simulation.payload }

    var transfer: GemTransferData { state.transfer }

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
        rowContents.indices.map { rowContents[$0].item(at: $0) }
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(header: confirmation.header(load: state.load), currency: confirmation.currency)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarnings)
        case let .row(index):
            ConfirmRowViewModel(
                content: rowContents[index],
                onSelectAddress: { [weak self] in self?.onSelectAddress($0) },
            )
        case .verification:
            ConfirmVerificationViewModel(infoAction: onSelectVerificationInfo)
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
        guard case let .confirm(confirmError) = error,
              let info = confirmation.errorInfo(error: confirmError, metadata: state.metadata) else { return }
        isPresentingSheet = .info(ConfirmInfoSheetBuilder.build(
            for: info,
            networkFeeBuyAmount: Int(confirmation.insufficientNetworkFeeBuyAmount()),
            onGetAsset: { [weak self] asset, buyAmount in self?.onSelectGetAsset(asset, buyAmount: buyAmount) },
        ))
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(.networkFee(state.feeAsset))
    }

    public func fieldModels(for fields: [GemSimulationPayloadRow]) -> [SimulationPayloadFieldViewModel] {
        payloadModel.fieldModels(
            for: fields,
            explorerLink: { explorerLink(chain: transfer.chain, address: $0) },
            onOpenURL: { [weak self] in self?.isPresentingSheet = .url($0) },
            onSelectAddress: { [weak self] address in
                guard let self else { return }
                onSelectAddress(ChainAddress(chain: request.data.chain, address: address))
            },
        )
    }

    func onSelectPayloadDetails() {
        isPresentingSheet = .payloadDetails
    }

    func onSelectPaymentAsset() {
        guard state.screen.phase != .loading, let invoice = transfer.invoice else { return }
        isPresentingSheet = .paymentAsset(.payment(invoice.quotes.map { AssetId(core: $0.assetId) }))
    }

    public func selectPaymentAsset(_ asset: Asset) {
        isPresentingSheet = nil
        guard asset.id != transfer.asset.id || state.verification != nil else { return }
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

    func onSelectAddress(_ chainAddress: ChainAddress) {
        isPresentingSheet = .addressDetails(chainAddress)
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
            state = try await ConfirmTransferState(confirmation.state(), screen: state.screen)
            let load = try await confirmation.load(options: preloadSelection.loadOptions)
            state = ConfirmTransferState(load, screen: state.screen.onLoaded(load: load))
        } catch let error as GemConfirmError {
            guard !Task.isCancelled else { return }
            state.transfer = confirmation.transfer()
            state.screen = state.screen.onLoadFailed(error: error)
            debugLog("confirm load error: \(error)")
        } catch {
            debugLog("confirm load error: \(error)")
        }
    }

    private func onStateChange(state: ConfirmTransferState) {
        let screen = state.screen
        button = screen.button()
        feeRow = screen.feeRow()
        feeRates = state.feeRateRows(selection: feeSelection)
        rowContents = confirmation.rowContents(addressName: state.addressName?.toGem())
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
            } catch let error as GemConfirmError {
                state.screen = state.screen.onExecuteFailed(error: error)
                isPresentingAlertMessage = AlertMessage(title: Localized.Errors.transferError, message: error.display().localizedDescription)
                debugLog("confirm transaction error: \(error)")
            } catch {
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

    public func acquireSwapPair(to asset: Asset) -> GemSwapPairSelection {
        confirmation.acquireSwapPair(feeAssetId: state.feeAsset.id.identifier, assetId: asset.id.identifier)
    }

    public var assetAcquisitionWallet: Wallet {
        wallet
    }
}

// MARK: - Confirm

extension ConfirmTransferSceneViewModel {
    func explorerLink(chain: Chain, address: String) -> BlockExplorerLink {
        confirmation.explorerLink(chain: chain, address: address)
    }

    func submit(request: ConfirmTransferRequest) async throws -> GemSubmitResult {
        let result: GemSubmitResult
        do {
            result = try await confirmation.submit()
        } catch let GemConfirmError.Broadcast(hashes, msg) {
            hashes.forEach { request.delegate?(.success($0)) }
            throw GemConfirmError.Broadcast(hashes: hashes, msg: msg)
        }
        switch result {
        case let .signed(data, _):
            data.forEach { request.delegate?(.success($0)) }
        case let .sent(hashes, _):
            hashes.forEach { request.delegate?(.success($0)) }
        }
        return result
    }
}
