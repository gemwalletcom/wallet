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
public final class SetPriceAlertSceneViewModel {
    private let asset: Primitives.Asset
    private let service: any GemPriceAlertServiceProtocol
    private let onComplete: StringAction
    private let currency: Primitives.Currency
    private let currencyFormatter: CurrencyFormatter

    private var session: GemPriceAlertSession
    private var amounts: [SetPriceAlertType: String] = [:]
    var isPresentingAlertMessage: AlertMessage?

    public let assetQuery: ObservableQuery<AssetQuery>
    var assetData: Primitives.AssetData {
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
        session = service.newAlertSession(assetId: asset.id.identifier, format: NumberInput.format())
        assetQuery = ObservableQuery(AssetQuery(walletId: walletId, assetId: asset.id), initialValue: .with(asset: asset))
    }

    var viewState: GemPriceAlertViewState {
        pricedSession.viewState()
    }

    var type: SetPriceAlertType {
        get { SetPriceAlertType(notificationType: session.notificationType.toPrimitives()) }
        set { session = session.onType(notificationType: newValue.notificationType.toGem()).onInput(input: NumberInput.double(amounts[newValue, default: .empty])) }
    }

    var amount: String {
        get { amounts[type, default: .empty] }
        set {
            amounts[type] = newValue
            session = session.onInput(input: NumberInput.double(newValue))
        }
    }

    func suggestions(_ viewState: GemPriceAlertViewState) -> [PriceSuggestion] {
        let values = switch type {
        case .price: viewState.priceSuggestions
        case .percentage: viewState.percentageSuggestions
        }
        return values.map { PriceSuggestion(title: $0.label.text(), inputValue: $0.inputText) }
    }

    func directionTitle(_ viewState: GemPriceAlertViewState) -> String {
        viewState.prompt.title
    }

    func confirmButtonState(_ viewState: GemPriceAlertViewState) -> ButtonState {
        if viewState.isSaving {
            return .loading(showProgress: true)
        }
        return viewState.canConfirm ? .normal : .disabled
    }

    func onSelectSuggestion(_ suggestion: some SuggestionViewable) {
        amount = suggestion.inputValue
    }

    func currencyInputConfig(_ viewState: GemPriceAlertViewState) -> any CurrencyInputConfigurable {
        SetPriceAlertCurrencyInputConfig(
            input: viewState.input,
            secondaryText: viewState.currentPriceText?.text ?? .empty,
            formatter: currencyFormatter,
            onTapActionButton: toggleAlertDirection,
        )
    }

    var assetRow: GemAssetItemRow {
        assetListRow(data: assetData.toGem(), currency: currency.toGem(), scope: .total, style: GemSelectAssetType.priceAlert.flow().rowStyle)
    }

    // MARK: - Private

    private var completeMessage: String {
        viewState.savedMessage?.text ?? .empty
    }

    private var pricedSession: GemPriceAlertSession {
        session.onPrice(currentPrice: assetData.price?.price, priceChange: assetData.price?.priceChangePercentage24h)
    }

    private func priceAlert() -> Primitives.PriceAlert? {
        pricedSession.alert().map { $0.toPrimitives() }
    }

    private func toggleAlertDirection() {
        let direction: Primitives.PriceAlertDirection = switch session.selectedDirection.toPrimitives() {
        case .up: .down
        case .down: .up
        }
        session = session.onDirection(selectedDirection: direction.toGem())
    }
}

// MARK: - Business logic

extension SetPriceAlertSceneViewModel {
    func setPriceAlert() async {
        guard let alert = priceAlert() else { return }
        session = session.onSaving(isSaving: true)
        do {
            try await service.enable(priceAlert: alert)
            onComplete?(completeMessage)
        } catch {
            isPresentingAlertMessage = AlertMessage(error: error)
        }
        session = session.onSaving(isSaving: false)
    }
}
