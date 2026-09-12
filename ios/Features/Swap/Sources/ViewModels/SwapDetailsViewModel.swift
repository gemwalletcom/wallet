// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemSwapRate
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapPriceImpact
import struct Gemstone.SwapQuote
import Localization
import Primitives
import PrimitivesComponents

@Observable
public final class SwapDetailsViewModel {
    private let valueFormatter = ValueFormatter(style: .auto)
    private let percentSignLessFormatter = PercentFormatter.unsigned

    let state: StateViewType<[SwapProviderItem]>
    private let fromAssetPrice: AssetPriceValue
    private let toAssetPrice: AssetPriceValue
    private let providerViewModel: SwapProviderViewModel
    private let selectedQuote: Gemstone.SwapQuote
    private let slippage: SwapSlippage
    private let rate: GemSwapRate?
    private var isRateInverse = false
    private let priceViewModel: PriceViewModel
    private let isProviderSelectionEnabled: Bool
    private let swapPriceImpact: SwapPriceImpact?
    private let minReceiveValue: BigInt
    private let etaSeconds: UInt32?
    private let swapProviderSelectAction: ((SwapperQuote) -> Void)?

    public init(
        state: StateViewType<[SwapProviderItem]> = .data([]),
        fromAssetPrice: AssetPriceValue,
        toAssetPrice: AssetPriceValue,
        selectedQuote: Gemstone.SwapQuote,
        slippage: SwapSlippage,
        rate: GemSwapRate?,
        currency: String,
        isProviderSelectionEnabled: Bool = true,
        swapPriceImpact: SwapPriceImpact?,
        minReceiveValue: BigInt,
        etaSeconds: UInt32?,
        swapProviderSelectAction: ((SwapperQuote) -> Void)? = nil,
    ) {
        self.state = state
        self.fromAssetPrice = fromAssetPrice
        self.toAssetPrice = toAssetPrice
        providerViewModel = SwapProviderViewModel(providerData: selectedQuote.providerData)
        self.selectedQuote = selectedQuote
        self.slippage = slippage
        self.rate = rate
        priceViewModel = PriceViewModel(price: toAssetPrice.price, currencyCode: currency)
        self.isProviderSelectionEnabled = isProviderSelectionEnabled
        self.swapPriceImpact = swapPriceImpact
        self.minReceiveValue = minReceiveValue
        self.etaSeconds = etaSeconds
        self.swapProviderSelectAction = swapProviderSelectAction
    }

    // MARK: - Provider

    var providerText: String {
        providerViewModel.providerText
    }

    var providerImage: AssetImage {
        providerViewModel.providerImage
    }

    var selectedProviderItem: SwapProviderItem {
        SwapProviderItem(
            asset: toAssetPrice.asset,
            swapQuote: selectedQuote,
            selectedProvider: nil,
            priceViewModel: priceViewModel,
            valueFormatter: valueFormatter,
        )
    }

    var allowSelectProvider: Bool {
        isProviderSelectionEnabled && state.value.or([]).count > 1
    }

    var swapProvidersViewModel: SwapProvidersViewModel {
        SwapProvidersViewModel(state: state.map { .plain($0) })
    }

    // MARK: - Estimation

    var swapEstimationField: ListItemField? {
        guard let etaSeconds else { return nil }
        let estimationTime = EstimatedConfirmationFormatter().string(seconds: etaSeconds)
        guard estimationTime.isEmpty == false else { return nil }
        return ListItemField(title: Localized.Swap.EstimatedTime.title, value: estimationTime)
    }

    // MARK: - Rate

    var rateTitle: String {
        Localized.Buy.rate
    }

    var rateText: String? {
        rate.map { AssetRateViewModel(rate: $0).text(isInverse: isRateInverse) }
    }

    // MARK: - Price Impact

    var highImpactWarningTitle: String {
        priceImpactModel.highImpactWarningTitle
    }

    var priceImpactModel: PriceImpactViewModel {
        PriceImpactViewModel(fromAssetPrice: fromAssetPrice, swapPriceImpact: swapPriceImpact)
    }

    var shouldShowPriceImpactInDetails: Bool {
        priceImpactModel.showsInSummary
    }

    var priceImpactValue: String? {
        priceImpactModel.value?.value
    }

    // MARK: - Slippage

    var slippageField: ListItemField {
        let value: String = switch slippage {
        case .auto: Localized.Swap.slippageAuto
        case let .manual(bps): percentSignLessFormatter.string((Double(bps) / 100).rounded(toPlaces: 2))
        }
        return ListItemField(title: Localized.Swap.slippage, value: value)
    }

    // MARK: - Min receive

    var minReceiveField: ListItemField {
        ListItemField(
            title: Localized.Swap.minReceive,
            value: valueFormatter.string(minReceiveValue, asset: toAssetPrice.asset),
        )
    }

    var fromAsset: Asset {
        fromAssetPrice.asset
    }

}

// MARK: - Actions

extension SwapDetailsViewModel {
    func switchRateDirection() {
        isRateInverse.toggle()
    }

    func onFinishSwapProviderSelection(item: [SwapProviderItem]) {
        guard let quote = item.first?.swapperQuote else { return }
        swapProviderSelectAction?(quote)
    }
}
