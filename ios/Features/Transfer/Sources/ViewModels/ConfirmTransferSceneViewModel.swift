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
import struct Gemstone.GemConfirmSimulationState
import struct Gemstone.GemConfirmViewState
import struct Gemstone.GemFeeRateRows
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
            simulation: ConfirmSimulationState(result: request.simulation),
            screen: confirmation.screen(),
        )
        self.loadOptions = loadOptions
        self.state = state
        viewState = confirmation.viewState(screen: state.screen, addressName: state.addressName?.toGem())
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

    var simulationWarnings: [GemListRow] {
        state.simulation.warnings
    }

    public var primaryPayloadFields: [GemSimulationPayloadRow] { state.simulation.primaryFields }
    public var secondaryPayloadFields: [GemSimulationPayloadRow] { state.simulation.secondaryFields }

    var transfer: GemTransferData { state.transfer }

    var confirmButtonModel: ConfirmButtonViewModel {
        ConfirmButtonViewModel(
            button: viewState.button,
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
            selection: loadOptions.feeSelection,
            feeRates: viewState.feeRates,
            feeAssetPrice: state.metadata?.feePrice,
            feeAmount: state.fee?.value,
            additionalFees: state.fee?.additionalFees ?? [],
            feeAssets: state.feeAssets.map { $0.feeAssetItem(currency: confirmation.currency) },
            showsFeeAssets: state.load?.showsFeeAssets() ?? false,
            onSelect: { [weak self] in self?.changeFeeSelection($0) },
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
            primaryPayloadFields.isEmpty ? nil : ListSection(type: .payload, [.payload]),
            balanceChangeModels.isEmpty ? nil : ListSection(type: .balanceChanges, balanceChangeModels.indices.map(ConfirmTransferItem.balanceChange)),
            ListSection(type: .fee, [state.verification == nil ? .networkFee : .verification]),
            ListSection(type: .error, [.error]),
        ].compactMap(\.self)
    }

    private var detailItems: [ConfirmTransferItem] {
        viewState.rowContents.indices.map { viewState.rowContents[$0].item(at: $0) }
    }

    public func itemModel(for item: ConfirmTransferItem) -> any ItemModelProvidable<ConfirmTransferItemModel> {
        switch item {
        case .header:
            ConfirmHeaderViewModel(header: confirmation.header(screen: state.screen), currency: confirmation.currency)
        case .warnings:
            ConfirmTransferItemModel.warnings(simulationWarnings)
        case let .row(index):
            ConfirmRowViewModel(
                content: viewState.rowContents[index],
                onSelectAddress: { [weak self] in self?.onSelectAddress($0) },
            )
        case .verification:
            ConfirmVerificationViewModel(infoAction: onSelectVerificationInfo)
        case .details:
            detailsViewModel
        case .payload:
            ConfirmTransferItemModel.payload(fieldModels(for: primaryPayloadFields))
        case let .balanceChange(index):
            ConfirmTransferItemModel.balanceChange(balanceChangeModels[index])
        case .networkFee:
            ConfirmNetworkFeeViewModel(
                feeRow: viewState.feeRow,
                feeModel: feeModel,
                infoAction: onSelectNetworkFeeInfo,
            )
        case .error:
            ConfirmErrorViewModel(
                error: state.loadError,
                onSelectListError: onSelectListError,
            )
        }
    }
}

// MARK: - Business Logic

extension ConfirmTransferSceneViewModel {
    func onSelectListError(error: GemConfirmError) {
        guard let info = confirmation.errorInfo(error: error) else { return }
        isPresentingSheet = .info(ConfirmInfoSheetBuilder.build(
            for: info,
            onGetAsset: { [weak self] asset, acquire in self?.onSelectGetAsset(asset, acquire: acquire) },
        ))
    }

    func onSelectNetworkFeeInfo() {
        isPresentingSheet = .info(.networkFee(state.feeAsset))
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
        let assetIds = viewState.rowContents.lazy.compactMap { content -> [String]? in
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
            let load = try await confirmation.load(options: loadOptions)
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
        viewState = confirmation.viewState(screen: state.screen, addressName: state.addressName?.toGem())
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
