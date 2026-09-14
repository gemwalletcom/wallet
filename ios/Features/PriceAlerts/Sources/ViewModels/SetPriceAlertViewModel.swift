// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Gemstone
import GemstonePrimitives
import struct Gemstone.GemPriceAlertSession
import struct Gemstone.GemPriceAlertViewState
import protocol Gemstone.GemPriceAlertServiceProtocol
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
        currencyFormatter = CurrencyFormatter(currencyCode: service.getCurrency())
        self.onComplete = onComplete
        state = SetPriceAlertViewModelState()
        assetQuery = ObservableQuery(AssetRequest(walletId: walletId, assetId: asset.id), initialValue: .with(asset: asset))
    }

    func percentageSuggestions(for price: Primitives.Price?) -> [PercentageSuggestion] {
        viewState(price: price).percentageSuggestions.map { PercentageSuggestion(value: $0.asInt) }
    }

    func priceSuggestions(for price: Primitives.Price?) -> [PriceSuggestion] {
        viewState(price: price).priceSuggestions.map {
            PriceSuggestion(title: currencyFormatter.string($0), value: $0)
        }
    }

    func onSelectSuggestion(_ suggestion: some SuggestionViewable) {
        state.amount = suggestion.inputValue
    }

    private var session: GemPriceAlertSession {
        service.newAlertSession(assetId: asset.id.identifier)
            .onType(notificationType: state.type.notificationType.map())
            .onDirection(selectedDirection: state.selectedDirection.map())
            .onInput(input: amountValue)
            .onPrice(currentPrice: assetData.price?.price)
            .onSaving(isSaving: isSaving)
    }

    private func viewState(price: Primitives.Price?) -> GemPriceAlertViewState {
        session.onPrice(currentPrice: price?.price).viewState()
    }

    var alertDirection: Primitives.PriceAlertDirection? {
        session.viewState().direction.map { $0.map() }
    }

    var alertDirectionTitle: String {
        switch state.type {
        case .price:
            switch alertDirection {
            case .up: Localized.PriceAlerts.SetAlert.priceOver
            case .down: Localized.PriceAlerts.SetAlert.priceUnder
            case .none: Localized.PriceAlerts.SetAlert.setTargetPrice
            }
        case .percentage:
            switch state.selectedDirection {
            case .up: Localized.PriceAlerts.SetAlert.priceIncreasesBy
            case .down: Localized.PriceAlerts.SetAlert.priceDecreasesBy
            }
        }
    }

    var isEnabledConfirmButton: Bool {
        session.viewState().canConfirm
    }

    var confirmButtonState: ButtonState {
        if isSaving {
            return .loading(showProgress: true)
        }
        return isEnabledConfirmButton ? .normal : .disabled
    }

    func currencyInputConfig(for assetData: AssetData) -> any CurrencyInputConfigurable {
        SetPriceAlertCurrencyInputConfig(
            type: state.type,
            alertDirection: state.selectedDirection,
            assetData: assetData,
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
                currencyCode: currencyFormatter.currencyCode,
            ),
            row: GemSelectAssetType.priceAlert.flow().row,
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
        guard let amountValue else { return .empty }
        let amount: String = switch state.type {
        case .price: currencyFormatter.string(amountValue)
        case .percentage: "\(amountValue)%"
        }
        let message = [alertDirectionTitle.lowercased(), amount].joined(separator: " ")
        return Localized.PriceAlerts.addedFor(message)
    }

    private func priceAlert() -> Primitives.PriceAlert? {
        session.alert().map { $0.map() }
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
