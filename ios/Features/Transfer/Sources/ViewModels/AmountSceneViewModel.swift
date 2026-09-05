// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import GemstoneServices
import Formatters
import Foundation
import protocol Gemstone.GemAmountServiceProtocol
import GemstoneFormatters
import InfoSheet
import Localization
import Perpetuals
import GemstonePrimitives
import Primitives
import PrimitivesComponents
import Store
import Style
import Validators
import struct Gemstone.GemAmountInput
import struct Gemstone.GemTransferData

@MainActor
@Observable
public final class AmountSceneViewModel {
    private let service: any GemAmountServiceProtocol
    private let wallet: Wallet
    private let onTransferAction: TransferDataAction

    private let formatter = ValueFormatter(style: .full)
    private let amountFormatter = ValueFormatter.auto
    private let valueConverter = AssetValueConverter(formatter: .auto)
    let currencyFormatter: CurrencyFormatter

    public let provider: AmountDataProvider

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    public var transferState: StateViewType<GemTransferData> = .noData
    var amountInputModel: InputValidationViewModel
    public var isPresentingSheet: AmountSheetType?

    private var amountInputType: AmountInputType = .asset {
        didSet { amountInputModel.update(validators: inputValidators) }
    }

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
        amountInputModel = InputValidationViewModel(mode: .onDemand, validators: [])
        amountInputModel.update(validators: inputValidators)

        if let amount = provider.prefilledAmount {
            amountInputModel.update(text: amount)
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
        guard let reservedFee = input.reservedFee, amountInputModel.text == maxBalance else { return nil }
        return Localized.Transfer.reservedFees(formatter.string(reservedFee, asset: asset))
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
            sceneType: provider.amountType,
            canSwitchInputType: provider.gemAmountType.canSwitchInputType(),
            inputType: amountInputType,
            asset: asset,
            currencyFormatter: currencyFormatter,
            numberSanitizer: NumberSanitizer(),
            secondaryText: secondaryText,
            onTapActionButton: onSelectInputButton,
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
        amountInputModel.update(validators: inputValidators)
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
        amountInputModel.update(validators: inputValidators)
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
        guard let transferError = error as? TransferError,
              case let .minimumAmount(asset, required) = transferError
        else {
            return nil
        }
        return { [weak self] in
            guard let self else { return }
            isPresentingSheet = .infoAction(.minimumAmount(asset, required: required, action: onSelectBuy))
        }
    }
}

private extension AmountSceneViewModel {
    func setMax() {
        amountInputType = .asset
        amountInputModel.update(text: maxBalance)
    }

    var input: GemAmountInput {
        provider.input(from: assetData)
    }

    var maxBalance: String {
        formatter.string(input.maxValue, decimals: asset.decimals.asInt)
    }

    func cleanInput() {
        amountInputModel.text = .empty
        amountInputModel.update(validators: inputValidators)
    }

    func load() async {
        do {
            transferState = .loading
            let value = try amountTransferValue
            let transfer = try await provider.makeTransferData(value: value, useMaxAmount: value == input.maxValue)
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

    var inputValidators: [any TextValidator] {
        let source: AmountValidator.Source = switch amountInputType {
        case .asset: .asset
        case .fiat: .fiat(price: assetData.price?.mapToAssetPrice(assetId: asset.id), converter: valueConverter)
        }
        return [
            .amount(
                source: source,
                decimals: asset.decimals.asInt,
                validators: [
                    AmountValueValidator(type: provider.gemAmountType, asset: asset, balance: assetData.balance),
                ],
            ),
        ]
    }

    var amountTransferValue: BigInt {
        get throws {
            switch amountInputType {
            case .asset: try formatter.inputNumber(from: amountInputModel.text, decimals: asset.decimals.asInt)
            case .fiat: amountValue
            }
        }
    }

    var amountValue: BigInt {
        guard let price = assetData.price else { return .zero }
        return (try? valueConverter.convertToDisplayedAmount(
            fiatValue: amountInputModel.text,
            price: price.mapToAssetPrice(assetId: asset.id),
            decimals: asset.decimals.asInt,
        )) ?? .zero
    }

    var fiatValue: Decimal {
        guard let price = assetData.price else { return .zero }
        return (try? valueConverter.convertToFiat(
            amount: amountInputModel.text,
            price: price.mapToAssetPrice(assetId: asset.id),
            decimals: asset.decimals.asInt,
        )).or(.zero)
    }

    var secondaryText: String {
        switch amountInputType {
        case .asset: currencyFormatter.string(fiatValue.doubleValue)
        case .fiat: amountFormatter.string(amountValue, asset: asset)
        }
    }
}
