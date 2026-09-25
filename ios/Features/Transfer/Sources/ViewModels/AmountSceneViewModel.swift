// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAmountEntry
import enum Gemstone.GemAmountError
import struct Gemstone.GemAmountInput
import enum Gemstone.GemAmountInputType
import protocol Gemstone.GemAmountServiceProtocol
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

@MainActor
@Observable
public final class AmountSceneViewModel {
    private let service: any GemAmountServiceProtocol
    private let wallet: Wallet
    private let onTransferAction: TransferDataAction

    let currencyFormatter: CurrencyFormatter
    private let currency: Currency

    public let provider: AmountDataProvider

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
        provider = .make(from: input, service: service, stakeService: stakeService)
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: input.asset.id), initialValue: .with(asset: input.asset))
        let amountInput = provider.input(from: assetQuery.value)
        self.input = amountInput
        entry = provider.entry(from: assetQuery.value, input: amountInput, inputType: .asset, text: .empty, currency: currency)
        amountInputModel = InputValidationViewModel()
    }

    public var asset: Asset {
        provider.asset
    }

    var title: String {
        provider.title
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
        if case let .transfer(transfer) = provider {
            return AssetIdViewModel(assetId: transfer.displayAsset.id).assetImage
        }
        return AssetIdViewModel(assetId: asset.id).assetImage
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
            canSwitchInputType: provider.gemAmountType.canSwitchInputType(),
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
        isPresentingSheet = .infoAction(.stakingReservedFees(image: assetImage))
    }

    func onSelectBuy() {
        guard let address = try? wallet.account(for: asset.chain).address else { return }
        let assetAddress = AssetAddress(asset: asset, address: address)
        isPresentingSheet = .fiatConnect(assetAddress: assetAddress, wallet: wallet)
    }

    func onSelectLeverage() {
        guard case let .perpetual(perpetual) = provider,
              let selection = perpetual.leverageSelection else { return }
        isPresentingSheet = .leverageSelector(selection: selection)
    }

    func onSelectAutoclose() {
        guard case let .perpetual(perpetual) = provider else { return }
        let amount = NumberInput.double(amountInputModel.text) ?? .zero
        isPresentingSheet = .autoclose(perpetual.makeAutocloseData(size: amount))
    }

    public func onAutocloseComplete(_ selection: AutocloseSelection) {
        if case let .perpetual(perpetual) = provider {
            perpetual.updateAutoclose(takeProfit: selection.takeProfit, stopLoss: selection.stopLoss)
        }
        isPresentingSheet = nil
    }

    func onChangeResource(_: Resource, _ resource: Resource) {
        if case let .stake(stake) = provider {
            stake.select(resource)
        }
        cleanInput()
    }

    public func onChangeLeverage(_: LeverageOption, _: LeverageOption) {
        refreshEntry()
        if case let .perpetual(perpetual) = provider {
            perpetual.onChangeLeverage()
        }
    }

    public func onValidatorSelected(_ row: GemValidatorRow) {
        guard case let .stake(stake) = provider else { return }
        stake.select(row)
        refreshEntry()
    }

    func infoAction(for error: Error) -> (() -> Void)? {
        guard let topic = (error as? GemAmountError)?.display().info() else {
            return nil
        }
        return { [weak self] in
            guard let self else { return }
            isPresentingSheet = .infoAction(InfoSheetType(topic: topic, assetImage: assetImage, buyAction: onSelectBuy))
        }
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
        input = provider.input(from: assetData)
        entry = provider.entry(from: assetData, input: input, inputType: amountInputType, text: NumberInput.plain(amountInputModel.text), currency: currency)
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
            let transfer = try await provider.makeTransferData(value: value, useMaxAmount: entry.isMax)
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
}
