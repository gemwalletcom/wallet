// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Gemstone
import protocol Gemstone.GemPreferencesServiceProtocol
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
    private let priceAlertService: any GemPriceAlertServiceProtocol
    private let onComplete: StringAction
    private let preferencesService: any GemPreferencesServiceProtocol
    private let currencyFormatter: CurrencyFormatter
    private let numericFormatter = NumericFormatter()
    private let priceAlertFormatter = PriceAlertFormatter()
    private let suggestionOffsetPercent: Double = 5

    var state: SetPriceAlertViewModelState

    public let assetQuery: ObservableQuery<AssetRequest>
    var assetData: AssetData {
        assetQuery.value
    }

    public init(
        walletId: Primitives.WalletId,
        asset: Primitives.Asset,
        priceAlertService: any GemPriceAlertServiceProtocol,
        preferencesService: any GemPreferencesServiceProtocol,
        price: Double? = nil,
        onComplete: StringAction,
    ) {
        self.asset = asset
        self.priceAlertService = priceAlertService
        self.preferencesService = preferencesService
        currencyFormatter = CurrencyFormatter(currencyCode: preferencesService.currencyCode)
        self.onComplete = onComplete
        state = SetPriceAlertViewModelState(price: price)
        assetQuery = ObservableQuery(AssetRequest(walletId: walletId, assetId: asset.id), initialValue: .with(asset: asset))
    }

    func percentageSuggestions(for price: Primitives.Price?) -> [PercentageSuggestion] {
        guard let currentPrice = price?.price else { return [] }
        return priceAlertFormatter.percentageSuggestions(price: currentPrice).map {
            PercentageSuggestion(value: $0.asInt)
        }
    }

    func priceSuggestions(for price: Primitives.Price?) -> [PriceSuggestion] {
        guard let currentPrice = price?.price else { return [] }
        return priceAlertFormatter.roundedValues(price: currentPrice, byPercent: suggestionOffsetPercent).map {
            PriceSuggestion(
                title: currencyFormatter.string($0),
                value: $0,
            )
        }
    }

    func onSelectSuggestion(_ suggestion: some SuggestionViewable) {
        state.amount = suggestion.inputValue
    }

    var alertDirection: Primitives.PriceAlertDirection? {
        priceAlertFormatter.alertDirection(
            notificationType: state.type.notificationType.json(),
            inputValue: amountValue,
            currentPrice: assetData.price?.price,
            selectedDirection: state.selectedDirection.json(),
        )
        .flatMap { try? Primitives.PriceAlertDirection($0) }
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
        alertDirection != nil
    }

    var confirmButtonState: ButtonState {
        isEnabledConfirmButton ? .normal : .disabled
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
            type: .price,
        )
    }

    func onChangeAlertType(_: SetPriceAlertType, type: SetPriceAlertType) {
        state.type = type
    }

    // MARK: - Private

    private var amountValue: Double? {
        numericFormatter.double(from: state.amount)
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

    private func priceAlert() -> Primitives.PriceAlert {
        let (price, pricePercentChange): (Double?, Double?) = switch state.type {
        case .price: (amountValue, nil)
        case .percentage: (nil, amountValue)
        }
        return Primitives.PriceAlert(
            assetId: asset.id,
            currency: preferencesService.currencyValue,
            price: price,
            pricePercentChange: pricePercentChange,
            priceDirection: alertDirection,
            lastNotifiedAt: .none,
        )
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
        let priceAlert = priceAlert()
        onComplete?(completeMessage)
        do {
            try await priceAlertService.enable(priceAlert: priceAlert)
        } catch {
            debugLog("Set price alert error: \(error.localizedDescription)")
        }
    }
}
