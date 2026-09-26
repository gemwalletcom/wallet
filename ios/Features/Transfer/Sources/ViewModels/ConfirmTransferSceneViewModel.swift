// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemAcquireAsset
import protocol Gemstone.GemConfirmationProtocol
import struct Gemstone.GemConfirmButton
import enum Gemstone.GemConfirmError
import enum Gemstone.GemConfirmFeeRow
import enum Gemstone.GemConfirmFeeSelection
import struct Gemstone.GemConfirmLoadOptions
import enum Gemstone.GemConfirmRowContent
import enum Gemstone.GemConfirmSection
import struct Gemstone.GemConfirmViewState
import struct Gemstone.GemFeeRateRows
import enum Gemstone.GemInfoAction
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import protocol Gemstone.GemPreferencesServiceProtocol
import struct Gemstone.GemSimulationPayloadRow
import enum Gemstone.GemSubmitResult
import struct Gemstone.GemTransferData
import struct Gemstone.SimulationResult
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import Swap
import SwiftUI
import WalletConnector

@Observable
@MainActor
public final class ConfirmTransferSceneViewModel {
    private(set) var loadOptions: GemConfirmLoadOptions

    var state: ConfirmTransferState {
        didSet { onStateChange(state: state) }
    }

    private(set) var viewState: GemConfirmViewState

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

        let loadOptions = confirmation.loadOptions()
        let state = ConfirmTransferState(
            transfer: request.data,
            screen: confirmation.screen(),
        )
        self.loadOptions = loadOptions
        self.state = state
        viewState = confirmation.viewState(screen: state.screen)
    }

    var payloadDetailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    var title: String {
        viewState.title.title
    }

    var progressMessage: String {
        Localized.Common.loading
    }

    var isConfirming: Bool {
        state.screen.phase == .confirming
    }

    var simulationWarnings: [GemListRow] {
        viewState.sections.lazy.compactMap { section -> [GemListRow]? in
            guard case let .warnings(rows) = section else { return nil }
            return rows
        }.first ?? []
    }

    public var primaryPayloadFields: [GemSimulationPayloadRow] { payload.primary }
    public var secondaryPayloadFields: [GemSimulationPayloadRow] { payload.secondary }

    private var payload: (primary: [GemSimulationPayloadRow], secondary: [GemSimulationPayloadRow]) {
        viewState.sections.lazy.compactMap { section -> ([GemSimulationPayloadRow], [GemSimulationPayloadRow])? in
            guard case let .payload(primary, secondary) = section else { return nil }
            return (primary, secondary)
        }.first ?? ([], [])
    }

    private var rowContents: [GemConfirmRowContent] {
        viewState.sections.lazy.compactMap { section -> [GemConfirmRowContent]? in
            guard case let .details(rows) = section else { return nil }
            return rows
        }.first ?? []
    }

    private var notice: GemListRow? {
        viewState.sections.lazy.compactMap { section -> GemListRow? in
            guard case let .notice(row) = section else { return nil }
            return row
        }.first
    }

    private var loadError: GemConfirmError? {
        viewState.sections.lazy.compactMap { section -> GemConfirmError? in
            guard case let .error(error) = section else { return nil }
            return error
        }.first
    }

    var transfer: GemTransferData { state.transfer }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            button: viewState.button,
            authentication: viewState.authentication,
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
        viewState.sections.lazy.compactMap { section -> [ConfirmBalanceChangeViewModel]? in
            guard case let .balanceChanges(changes) = section else { return nil }
            return changes.map(ConfirmBalanceChangeViewModel.init)
        }.first ?? []
    }

    public var feeModel: NetworkFeeSceneViewModel {
        NetworkFeeSceneViewModel(
            feeAsset: state.feeAsset,
            currency: confirmation.currency,
            selection: loadOptions.feeSelection,
            feeRates: viewState.feeRates,
            feeAssetPrice: state.metadata?.feePrice,
            feeAmount: state.fee?.value,
            fee: state.fee?.formatted,
            additionalFees: state.fee?.additionalFees ?? [],
            feeAssets: state.feeAssets.map(\.feeAssetItem),
            selectedFeeAsset: state.load.map { FeeAssetItem(asset: state.feeAsset, row: $0.feeAssetRow(currency: confirmation.currency.toGem()), isSelected: false) },
            showsFeeAssets: state.load?.showsFeeAssets() ?? false,
            onSelect: { [weak self] in self?.changeFeeSelection($0) },
            onSelectFeeAsset: { [weak self] in self?.selectFeeAsset($0) },
        )
    }
}

// MARK: - Sections

extension ConfirmTransferSceneViewModel {
    public var sections: [ListSection<ConfirmTransferItem>] {
        viewState.sections.map { section in
            switch section {
            case .header: ListSection(type: .header, [.header])
            case .notice: ListSection(type: .notice, [.notice])
            case let .details(rows): ListSection(type: .details, rows.indices.map { rows[$0].item(at: $0) })
            case .warnings: ListSection(type: .warnings, [.warnings])
            case .payload: ListSection(type: .payload, [.payload])
            case let .balanceChanges(changes): ListSection(type: .balanceChanges, changes.indices.map(ConfirmTransferItem.balanceChange))
            case .networkFee: ListSection(type: .fee, [.networkFee])
            case .verification: ListSection(type: .fee, [.verification])
            case .error: ListSection(type: .error, [.error])
            }
        }
    }

    public func itemModel(for item: ConfirmTransferItem) -> ConfirmTransferItemModel {
        switch item {
        case .header:
            .header(confirmation.header(screen: state.screen))
        case .notice:
            notice.map(ConfirmTransferItemModel.row) ?? .empty
        case .warnings:
            .warnings(simulationWarnings)
        case let .row(index):
            ConfirmRowViewModel(content: rowContents[index]).itemModel
        case .verification:
            verificationItem
        case .details:
            detailsViewModel.itemModel
        case .payload:
            .payload(fieldModels(for: primaryPayloadFields))
        case let .balanceChange(index):
            .balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                feeRow: viewState.feeRow,
                feeModel: feeModel,
                infoAction: onSelectNetworkFeeInfo,
            ).itemModel
        case .error:
            errorItem(loadError)
        }
    }

    private var verificationItem: ConfirmTransferItemModel {
        .verification(
            ListItemModel(
                title: Localized.Info.paymentVerificationTitle,
                subtitle: "",
                subtitleStyle: TextStyle(font: .body, color: Colors.orange),
                subtitleTagType: .image(Image(systemName: SystemImage.clockBadgeExclamationmark)),
                infoAction: onSelectVerificationInfo,
            ),
        )
    }

    private func errorItem(_ error: GemConfirmError?) -> ConfirmTransferItemModel {
        guard let error else { return .empty }
        return .error(
            title: Localized.Errors.errorOccurred,
            error: error,
            onInfoAction: error.display().hasInfoSheet() ? { [weak self] in self?.onSelectListError(error: error) } : nil,
        )
    }
}

// MARK: - Business Logic

extension ConfirmTransferSceneViewModel {
    func onSelectListError(error: GemConfirmError) {
        guard let info = confirmation.errorInfo(error: error) else { return }
        isPresentingSheet = .info(info.infoSheet)
    }

    func onInfo(_ topic: GemInfoTopic) {
        isPresentingSheet = .info(topic.infoSheet)
    }

    public func onInfoAction(_ action: GemInfoAction) {
        guard case let .acquire(asset, acquire) = action else { return }
        onSelectGetAsset(asset.toPrimitives(), acquire: acquire)
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(GemInfoTopic.networkFee(asset: state.feeAsset.toGem()).infoSheet)
    }

    public func fieldModels(for fields: [GemSimulationPayloadRow]) -> [SimulationPayloadFieldViewModel] {
        SimulationPayloadFieldViewModel.models(
            for: fields,
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
        let assetIds = rowContents.lazy.compactMap { content -> [String]? in
            guard case let .paymentAsset(_, selectable, assetIds) = content, selectable else { return nil }
            return assetIds
        }.first
        guard let assetIds else { return }
        isPresentingSheet = .paymentAsset(.payment(assetIds.map { AssetId(core: $0) }))
    }

    public func selectPaymentAsset(_ asset: Asset) {
        isPresentingSheet = nil
        loadOptions = loadOptions.onPaymentAsset(picked: asset.id.identifier, transfer: transfer)
    }

    func onSelectVerification() {
        guard let verification = viewState.verification, let url = URL(string: verification.url) else { return }
        isPresentingSheet = .paymentVerification(url)
    }

    func onSelectVerificationInfo() {
        isPresentingSheet = .info(GemInfoTopic.paymentVerification.infoSheet)
    }

    public func onPaymentVerified() {
        isPresentingSheet = nil
        Task { await load() }
    }

    public func onPaymentVerificationFailed() {
        isPresentingSheet = nil
        isPresentingAlertMessage = AlertMessage(message: Localized.Errors.errorOccurred)
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
            let load = try await confirmation.load(options: loadOptions)
            state = ConfirmTransferState(load, screen: state.screen.onLoaded(load: load))
        } catch let error as GemConfirmError {
            if case .Cancelled = error {
                return
            }
            guard !Task.isCancelled else { return }
            state.transfer = confirmation.transfer()
            state.screen = state.screen.onLoadFailed(error: error)
            debugLog("confirm load error: \(error)")
        } catch {
            debugLog("confirm load error: \(error)")
        }
    }

    private func onStateChange(state: ConfirmTransferState) {
        viewState = confirmation.viewState(screen: state.screen)
        guard state.screen.presentsSheet(), let error = state.loadError else { return }
        onSelectListError(error: error)
    }

    func changeFeeSelection(_ selection: GemConfirmFeeSelection) {
        loadOptions = loadOptions.onFeeSelection(selection: selection)
    }

    private func selectFeeAsset(_ assetId: AssetId) {
        loadOptions = loadOptions.onFeeAsset(picked: assetId.identifier, loadedFeeAsset: state.feeAsset.id.identifier)
    }
}

// MARK: - Private

extension ConfirmTransferSceneViewModel {
    private func onSelectGetAsset(_ asset: Asset, acquire: GemAcquireAsset) {
        switch acquire.flow {
        case .options:
            isPresentingSheet = .getAsset(asset, acquire: acquire)
        case .fiat:
            isPresentingSheet = .fiatConnect(
                assetAddress: AssetAddress(asset: asset, address: senderAddress),
                wallet: wallet,
                amount: acquire.buyAmount.map(Int.init),
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

    public var assetAcquisitionWallet: Wallet {
        wallet
    }
}

// MARK: - Confirm

extension ConfirmTransferSceneViewModel {
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
