// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemAmountEntry
import enum Gemstone.GemAmountError
import struct Gemstone.GemAmountInput
import enum Gemstone.GemAmountInputType
import protocol Gemstone.GemAmountServiceProtocol
import struct Gemstone.GemTransferData
import GemstonePrimitives
import GemstoneServices
import InfoSheet
import Localization
import Perpetuals
import Primitives
import PrimitivesComponents
import Store
import Style
import Validators

@MainActor
@Observable
public final class AmountSceneViewModel {
    private let service: any GemAmountServiceProtocol
    private let wallet: Wallet
    private let onTransferAction: TransferDataAction

    private let formatter = ValueFormatter(style: .full)
    private let amountFormatter = ValueFormatter.auto
    let currencyFormatter: CurrencyFormatter

    public let provider: AmountDataProvider

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    public var transferState: StateViewType<GemTransferData> = .noData
    var amountInputModel: InputValidationViewModel
    public var isPresentingSheet: AmountSheetType?
    private(set) var entry: GemAmountEntry
    private(set) var amountInputType: GemAmountInputType = .asset

    public init(
        input: AmountInput,
        wallet: Wallet,
        service: any GemAmountServiceProtocol,
        onTransferAction: TransferDataAction,
    ) {
        self.wallet = wallet
        self.service = service
        self.onTransferAction = onTransferAction
        currencyFormatter = CurrencyFormatter(type: .currency, currencyCode: service.getCurrency())
        provider = .make(from: input, service: service)
        assetQuery = ObservableQuery(AssetRequest(walletId: wallet.id, assetId: input.asset.id), initialValue: .with(asset: input.asset))
        entry = provider.entry(from: assetQuery.value, inputType: .asset, text: .empty)
        amountInputModel = InputValidationViewModel(mode: .manual)

        if let amount = provider.prefilledAmount {
            amountInputModel.text = amount
            refreshEntry()
        }
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
            return AssetViewModel(asset: transfer.displayAsset).assetImage
        }
        return AssetViewModel(asset: asset).assetImage
    }

    var assetName: String {
        asset.name
    }

    var balanceText: String {
        let value = ValueFormatter(style: .auto).string(
            input.availableValue,
            decimals: asset.decimals.asInt,
            currency: asset.symbol,
        )
        return Localized.Transfer.balance(value)
    }

    var actionButtonState: ButtonState {
        if transferState.isLoading { return .loading() }
        return amountInputModel.text.isNotEmpty && amountInputModel.isValid ? .normal : .disabled
    }

    var infoText: String? {
        guard let reservedFee = entry.reservedFee else { return nil }
        return Localized.Transfer.reservedFees(amountFormatter.string(reservedFee, asset: asset))
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
            numberSanitizer: NumberSanitizer(),
            secondaryText: secondaryText,
            onTapActionButton: onSelectInputButton,
            usesWholeAmounts: input.usesWholeAmounts,
        )
    }
}

extension AmountSceneViewModel {
    var shouldFocusOnAppear: Bool {
        canChangeValue
    }

    func onAppear() {
        if !canChangeValue {
            setMax()
        }
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
        amountInputType = amountInputType == .asset ? .fiat : .asset
        cleanInput()
    }

    func onSelectReservedFeesInfo() {
        isPresentingSheet = .infoAction(.stakingReservedFees(image: assetImage))
    }

    func onSelectLeverage() {
        guard case let .perpetual(perpetual) = provider,
              let selection = perpetual.leverageSelection else { return }
        isPresentingSheet = .leverageSelector(selection: selection)
    }

    func onSelectAutoclose() {
        guard case let .perpetual(perpetual) = provider else { return }
        let amount = NumericFormatter().double(from: amountInputModel.text) ?? .zero
        isPresentingSheet = .autoclose(perpetual.makeAutocloseData(size: amount))
    }

    public func onAutocloseComplete(_ selection: AutocloseSelection) {
        if case let .perpetual(perpetual) = provider {
            perpetual.updateAutoclose(takeProfit: selection.takeProfit, stopLoss: selection.stopLoss)
        }
        isPresentingSheet = nil
    }

    func onChangeResource(_: Resource, _: Resource) {
        cleanInput()
    }

    public func onChangeLeverage(_: LeverageOption, _: LeverageOption) {
        refreshEntry()
        if case let .perpetual(perpetual) = provider {
            perpetual.onChangeLeverage()
        }
    }

    public func onValidatorSelected(_ validator: DelegationValidator) {
        guard case let .stake(stake) = provider,
              case let .validator(state) = stake.selection else { return }
        state.selected = validator
        if !canChangeValue {
            setMax()
        }
    }

    func infoAction(for error: Error) -> (() -> Void)? {
        guard case let .BelowMinimum(asset, required)? = error as? GemAmountError else {
            return nil
        }
        return { [weak self] in
            guard let self else { return }
            isPresentingSheet = .infoAction(.minimumAmount(asset.map(), required: required, action: onSelectBuy))
        }
    }
}

private extension AmountSceneViewModel {
    func setMax() {
        let max = input.maxEntry()
        amountInputType = max.inputType
        amountInputModel.text = formatter.string(max.value, decimals: asset.decimals.asInt)
        refreshEntry()
    }

    func refreshEntry() {
        let text = amountInputModel.text
        entry = provider.entry(from: assetData, inputType: amountInputType, text: text.isEmpty ? text : formatter.plainInputNumber(text))
        amountInputModel.update(error: entryError)
    }

    var entryError: (any Error)? {
        switch entry.error {
        case .none: nil
        case .Zero: SilentValidationError()
        case let .some(error): error
        }
    }

    var input: GemAmountInput {
        provider.input(from: assetData)
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

    func onSelectBuy() {
        let senderAddress = (try? wallet.account(for: asset.chain).address) ?? ""
        let assetAddress = AssetAddress(asset: asset, address: senderAddress)
        isPresentingSheet = .fiatConnect(assetAddress: assetAddress, wallet: wallet)
    }

    var secondaryText: String {
        switch entry.equivalent {
        case let .fiat(amount)?: currencyFormatter.string(amount)
        case let .asset(value)?: amountFormatter.string(value, asset: asset)
        case nil:
            switch amountInputType {
            case .asset: currencyFormatter.string(.zero)
            case .fiat: amountFormatter.string(.zero, asset: asset)
            }
        }
    }
}
