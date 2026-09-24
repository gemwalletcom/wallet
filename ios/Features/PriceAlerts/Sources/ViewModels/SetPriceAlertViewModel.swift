// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Gemstone
import protocol Gemstone.GemPriceAlertServiceProtocol
import struct Gemstone.GemPriceAlertSession
import struct Gemstone.GemPriceAlertViewState
import GemstonePrimitives
import GemstoneServices
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style

@MainActor
@Observable
public final class SetPriceAlertViewModel {
    private let asset: Primitives.Asset
    private let service: any GemPriceAlertServiceProtocol
    private let onComplete: StringAction
    private let currency: Primitives.Currency
    private let currencyFormatter: CurrencyFormatter

    var state: SetPriceAlertViewModelState
    var isPresentingAlertMessage: AlertMessage?
    private var isSaving = false

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    public init(
        walletId: Primitives.WalletId,
        asset: Primitives.Asset,
        service: any GemPriceAlertServiceProtocol,
        onComplete: StringAction,
    ) {
        self.asset = asset
        self.service = service
        currency = service.getCurrency().toPrimitives()
        currencyFormatter = CurrencyFormatter(currencyCode: currency.rawValue)
        self.onComplete = onComplete
        state = SetPriceAlertViewModelState()
        assetQuery = ObservableQuery(AssetRequest(walletId: walletId, assetId: asset.id), initialValue: .with(asset: asset))
    }

    var viewState: GemPriceAlertViewState {
        session.viewState()
    }

    func suggestions(_ viewState: GemPriceAlertViewState) -> [PriceSuggestion] {
        let values = switch state.type {
        case .price: viewState.priceSuggestions
        case .percentage: viewState.percentageSuggestions
        }
        return values.map { PriceSuggestion(title: $0.text(), value: $0.value) }
    }

    func directionTitle(_ viewState: GemPriceAlertViewState) -> String {
        viewState.prompt.title
    }

    func confirmButtonState(_ viewState: GemPriceAlertViewState) -> ButtonState {
        if isSaving {
            return .loading(showProgress: true)
        }
        return viewState.canConfirm ? .normal : .disabled
    }

    func onSelectSuggestion(_ suggestion: some SuggestionViewable) {
        state.amount = suggestion.inputValue
    }

    private var session: GemPriceAlertSession {
        service.newAlertSession(assetId: asset.id.identifier)
            .onType(notificationType: state.type.notificationType.toGem())
            .onDirection(selectedDirection: state.selectedDirection.toGem())
            .onInput(input: amountValue)
            .onPrice(currentPrice: assetData.price?.price, priceChange: assetData.price?.priceChangePercentage24h)
            .onSaving(isSaving: isSaving)
    }

    var currencyInputConfig: any CurrencyInputConfigurable {
        SetPriceAlertCurrencyInputConfig(
            type: state.type,
            alertDirection: state.selectedDirection,
            currentPrice: viewState.currentPrice,
            formatter: currencyFormatter,
            onTapActionButton: toggleAlertDirection,
        )
    }

    func assetItemViewModel(for assetData: AssetData) -> ListAssetItemViewModel {
        ListAssetItemViewModel(
            showBalancePrivacy: .constant(false),
            assetDataModel: AssetDataViewModel(
                assetData: assetData,
                formatter: .short,
                currency: currency,
            ),
            rowStyle: GemSelectAssetType.priceAlert.flow().rowStyle,
        )
    }

    func onChangeAlertType(_: SetPriceAlertType, type: SetPriceAlertType) {
        state.type = type
    }

    // MARK: - Private

    private var amountValue: Double? {
        NumberInput.double(state.amount)
    }

    private var completeMessage: String {
        let viewState = viewState
        guard let savedValue = viewState.savedValue else { return .empty }
        let message = [directionTitle(viewState).lowercased(), savedValue.text()].joined(separator: " ")
        return Localized.PriceAlerts.addedFor(message)
    }

    private func priceAlert() -> Primitives.PriceAlert? {
        session.alert().map { $0.toPrimitives() }
    }

    private func toggleAlertDirection() {
        state.selectedDirection = switch state.selectedDirection {
        case .up: .down
        case .down: .up
        }
    }
}

// MARK: - Business logic

extension SetPriceAlertViewModel {
    func setPriceAlert() async {
        guard let alert = priceAlert() else { return }
        isSaving = true
        do {
            try await service.enable(priceAlert: alert)
            onComplete?(completeMessage)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
        isSaving = false
    }
}
