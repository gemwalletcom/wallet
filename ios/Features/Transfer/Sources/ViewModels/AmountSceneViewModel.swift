// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAmountEntry
import enum Gemstone.GemAmountError
import struct Gemstone.GemAmountInput
import enum Gemstone.GemAmountInputType
import enum Gemstone.GemAmountRequest
import protocol Gemstone.GemAmountServiceProtocol
import enum Gemstone.GemAmountType
import struct Gemstone.GemAssetBalance
import enum Gemstone.GemInfoAction
import enum Gemstone.GemInfoTopic
import protocol Gemstone.GemStakeServiceProtocol
import struct Gemstone.GemTransferData
import struct Gemstone.GemValidatorRow
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Perpetuals
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

@MainActor
@Observable
public final class AmountSceneViewModel {
    private let service: any GemAmountServiceProtocol
    private let wallet: Wallet
    private let onTransferAction: TransferDataAction

    let currencyFormatter: CurrencyFormatter
    private let currency: Currency

    public let asset: Asset
    public let stake: AmountStakeViewModel?
    public let perpetual: AmountPerpetualViewModel?
    private let baseRequest: GemAmountRequest

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    public var transferState: StateViewType<GemTransferData> = .noData
    var amountInputModel: InputValidationViewModel
    public var isPresentingSheet: AmountSheetType?
    private(set) var input: GemAmountInput
    private(set) var entry: GemAmountEntry
    private(set) var amountInputType: GemAmountInputType = .asset

    public init(
        input: AmountInput,
        wallet: Wallet,
        service: any GemAmountServiceProtocol,
        stakeService: any GemStakeServiceProtocol,
        onTransferAction: TransferDataAction,
    ) {
        self.wallet = wallet
        self.service = service
        self.onTransferAction = onTransferAction
        currency = service.getCurrency().toPrimitives()
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: currency.rawValue)
        asset = input.asset
        var stake: AmountStakeViewModel?
        var perpetual: AmountPerpetualViewModel?
        switch input.type {
        case let .transfer(recipient): baseRequest = .transfer(transfer: .send(payment: recipient))
        case .deposit: baseRequest = .transfer(transfer: .deposit)
        case .withdraw: baseRequest = .transfer(transfer: .withdraw)
        case let .earn(earnType): baseRequest = .earn(earnType: earnType)
        case let .stake(type):
            let model = AmountStakeViewModel(asset: input.asset, type: type, service: stakeService)
            stake = model
            baseRequest = model.request
        case let .perpetual(action):
            let model = AmountPerpetualViewModel(asset: input.asset, action: action, service: service)
            perpetual = model
            baseRequest = model.request
        }
        self.stake = stake
        self.perpetual = perpetual
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: input.asset.id), initialValue: .with(asset: input.asset))
        let request = stake?.request ?? perpetual?.request ?? baseRequest
        let amountInput = Self.input(request: request, asset: input.asset, assetData: assetQuery.value)
        self.input = amountInput
        entry = Self.entry(amountType: request.amountType(), asset: input.asset, assetData: assetQuery.value, input: amountInput, inputType: .asset, text: .empty, currency: currency)
        amountInputModel = InputValidationViewModel()
        perpetual?.onInfo = { [weak self] topic in
            self?.isPresentingSheet = .infoAction(topic.infoSheet)
        }
    }

    var request: GemAmountRequest {
        stake?.request ?? perpetual?.request ?? baseRequest
    }

    var amountType: GemAmountType {
        request.amountType()
    }

    var title: String {
        amountType.title().title
    }

    var earnProviderRow: GemValidatorRow? {
        guard case let .earn(_, provider) = amountType else { return nil }
        return provider
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var displayAsset: Asset {
        request.displayAsset(asset: asset.toGem()).toPrimitives()
    }

    var canChangeValue: Bool {
        input.canChangeValue
    }

    var isInputDisabled: Bool {
        !canChangeValue
    }

    var isBalanceViewEnabled: Bool {
        input.showsAssetBalance
    }

    var assetImage: AssetImage {
        AssetIdViewModel(assetId: displayAsset.id).assetImage
    }

    var assetName: String {
        asset.name
    }

    var balanceText: String {
        Localized.Transfer.balance(input.balance.text())
    }

    var actionButtonState: ButtonState {
        if transferState.isLoading {
            return .loading()
        }
        return entry.allowsConfirm() ? .normal : .disabled
    }

    var infoText: String? {
        guard let reservedFee = entry.reservedFee else { return nil }
        return Localized.Transfer.reservedFees(reservedFee.text())
    }

    var maxTitle: String {
        Localized.Transfer.max
    }

    public var continueTitle: String {
        Localized.Common.continue
    }

    public var isNextEnabled: Bool {
        actionButtonState == .normal
    }

    var inputConfig: any CurrencyInputConfigurable {
        AmountInputConfig(
            canSwitchInputType: amountType.canSwitchInputType(),
            inputType: amountInputType,
            asset: asset,
            currencyFormatter: currencyFormatter,
            numberFormat: NumberInput.format(),
            secondaryText: secondaryText,
            onTapActionButton: onSelectInputButton,
            usesWholeAmounts: input.usesWholeAmounts,
        )
    }
}

extension AmountSceneViewModel {
    func prefillAmount() {
        guard let prefill = input.prefill,
              let text = NumberInput.format().inputText(value: prefill.value.description, decimals: UInt32(asset.decimals)) else { return }
        amountInputType = prefill.inputType
        amountInputModel.text = text
        refreshEntry()
    }

    public func onChangeAssetBalance(_: AssetData, _: AssetData) {
        refreshEntry()
    }

    func onChangeAmountText(_: String, _: String) {
        refreshEntry()
    }

    public func onSelectNextButton() {
        Task {
            await load()
        }
    }

    func onSelectMaxButton() {
        setMax()
    }

    func onSelectInputButton() {
        amountInputType = amountInputType.toggled()
        cleanInput()
    }

    func onSelectReservedFeesInfo() {
        isPresentingSheet = .infoAction(GemInfoTopic.stakingReservedFees(asset: asset.toGem()).infoSheet)
    }

    func onSelectBuy() {
        guard let address = try? wallet.account(for: asset.chain).address else { return }
        let assetAddress = AssetAddress(asset: asset, address: address)
        isPresentingSheet = .fiatConnect(assetAddress: assetAddress, wallet: wallet)
    }

    func onSelectLeverage() {
        guard let selection = perpetual?.leverageSelection else { return }
        isPresentingSheet = .leverageSelector(selection: selection)
    }

    func onSelectAutoclose() {
        guard let perpetual else { return }
        let amount = NumberInput.double(amountInputModel.text) ?? .zero
        isPresentingSheet = .autoclose(perpetual.makeAutocloseData(size: amount))
    }

    public func onAutocloseComplete(_ selection: AutocloseSelection) {
        perpetual?.updateAutoclose(takeProfit: selection.takeProfit, stopLoss: selection.stopLoss)
        isPresentingSheet = nil
    }

    func onSelectResource(_ resource: Resource) {
        stake?.select(resource)
        cleanInput()
    }

    func resourceBinding(selected: Resource) -> Binding<Resource> {
        Binding(
            get: { selected },
            set: { [self] in onSelectResource($0) },
        )
    }

    public func onChangeLeverage(_: LeverageOption, _: LeverageOption) {
        refreshEntry()
        perpetual?.onChangeLeverage()
    }

    public func onValidatorSelected(_ row: GemValidatorRow) {
        guard let stake else { return }
        stake.select(row)
        refreshEntry()
    }

    func infoAction(for error: Error) -> (() -> Void)? {
        guard let topic = (error as? GemAmountError)?.display().info() else {
            return nil
        }
        return { [weak self] in
            self?.isPresentingSheet = .infoAction(topic.infoSheet)
        }
    }

    public func onInfoAction(_ action: GemInfoAction) {
        guard case .buy = action else { return }
        onSelectBuy()
    }
}

private extension AmountSceneViewModel {
    func setMax() {
        let max = input.maxEntry()
        guard let text = NumberInput.format().inputText(value: max.value.description, decimals: UInt32(asset.decimals)) else { return }
        amountInputType = max.inputType
        amountInputModel.text = text
        refreshEntry()
    }

    func refreshEntry() {
        input = Self.input(request: request, asset: asset, assetData: assetData)
        entry = Self.entry(amountType: amountType, asset: asset, assetData: assetData, input: input, inputType: amountInputType, text: NumberInput.plain(amountInputModel.text), currency: currency)
        amountInputModel.update(error: entry.error)
    }

    func cleanInput() {
        amountInputModel.text = .empty
        refreshEntry()
    }

    func load() async {
        guard let value = entry.value else { return }
        do {
            transferState = .loading
            let transfer = try await service.transferData(asset: asset.toGem(), request: request, value: value, useMaxAmount: entry.isMax)
            transferState = .noData
            onTransferAction?(transfer)
        } catch {
            transferState = .error(error)
            amountInputModel.update(error: error)
        }
    }

    var secondaryText: String {
        entry.equivalent.text()
    }

    static func input(request: GemAmountRequest, asset: Asset, assetData: AssetData) -> GemAmountInput {
        request.input(asset: asset.toGem(), balance: GemAssetBalance(assetData.balance, assetId: asset.id, isActive: assetData.metadata.isActive))
    }

    static func entry(amountType: GemAmountType, asset: Asset, assetData: AssetData, input: GemAmountInput, inputType: GemAmountInputType, text: String, currency: Currency) -> GemAmountEntry {
        amountType.entry(asset: asset.toGem(), input: input, price: assetData.price?.price, inputType: inputType, text: text, currency: currency.toGem())
    }
}
