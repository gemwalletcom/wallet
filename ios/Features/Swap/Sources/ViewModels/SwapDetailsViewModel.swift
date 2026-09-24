// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Foundation
import enum Gemstone.GemInfoTopic
import enum Gemstone.GemListRow
import struct Gemstone.GemSwapQuoteSummary
import struct Gemstone.GemSwapRate
import enum Gemstone.SwapProvider
import struct Gemstone.SwapProviderData
import func Gemstone.swapProviderRow
import struct Gemstone.SwapQuote
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

@Observable
public final class SwapDetailsViewModel {
    private let valueFormatter = ValueFormatter(style: .auto)

    let state: StateViewType<[SwapProviderItem]>
    private let fromAssetPrice: AssetPriceValue
    private let toAssetPrice: AssetPriceValue
    private let providerData: SwapProviderData
    private let summary: GemSwapQuoteSummary
    private let selectedQuote: Gemstone.SwapQuote
    private let slippagePercent: Double?
    private let rate: GemSwapRate?
    private var isRateInverse = false
    private let currency: String
    let allowSelectProvider: Bool
    private let minReceiveValue: BigInt
    private let etaSeconds: UInt32?
    private let swapProviderSelectAction: ((SwapProvider) -> Void)?

    public init(
        state: StateViewType<[SwapProviderItem]> = .data([]),
        fromAssetPrice: AssetPriceValue,
        toAssetPrice: AssetPriceValue,
        summary: GemSwapQuoteSummary,
        slippagePercent: Double?,
        currency: String,
        allowSelectProvider: Bool = true,
        swapProviderSelectAction: ((SwapProvider) -> Void)? = nil,
    ) {
        self.state = state
        self.fromAssetPrice = fromAssetPrice
        self.toAssetPrice = toAssetPrice
        providerData = summary.quote.providerData
        self.summary = summary
        selectedQuote = summary.quote
        self.slippagePercent = slippagePercent
        rate = summary.rate
        self.currency = currency
        self.allowSelectProvider = allowSelectProvider
        minReceiveValue = BigInt(summary.minReceiveValue)
        etaSeconds = summary.quote.etaInSeconds
        self.swapProviderSelectAction = swapProviderSelectAction
    }

    var detailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var detailRows: [GemListRow] {
        summary.detailRows(hasSelectedSlippage: slippagePercent != nil)
    }

    // MARK: - Provider

    var providerText: String {
        providerData.name
    }

    var providerImage: AssetImage {
        AssetImage(imageURL: .none, placeholder: providerData.provider.toPrimitives().image, chainPlaceholder: .none)
    }

    var selectedProviderItem: SwapProviderItem {
        SwapProviderItem(
            row: swapProviderRow(
                provider: selectedQuote.providerData.provider,
                title: selectedQuote.providerData.protocolName,
                toValue: selectedQuote.toValue,
                receiveAsset: toAssetPrice.asset.toGem(),
                receivePrice: toAssetPrice.price?.price,
                currency: Primitives.Currency(rawValue: currency)?.toGem() ?? Primitives.Currency.usd.toGem(),
                isSelected: false,
            ),
        )
    }

    var swapProvidersViewModel: SwapProvidersViewModel {
        SwapProvidersViewModel(state: state.map { .plain($0) })
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
        Localized.Swap.PriceImpactWarning.title
    }

    var highImpactWarningDescription: String? {
        summary.priceImpactRow?.warning?.text
    }

    var shouldShowPriceImpactInDetails: Bool {
        summary.priceImpactRow?.showsInSummary == true
    }

    var priceImpactValue: String? {
        summary.priceImpactRow?.value.text()
    }

    var priceImpactStyle: TextStyle {
        TextStyle(font: .callout, color: summary.priceImpactRow?.value.tone.color ?? Colors.gray)
    }
}

// MARK: - Actions

extension SwapDetailsViewModel {
    func switchRateDirection() {
        isRateInverse.toggle()
    }

    func onFinishSwapProviderSelection(item: [SwapProviderItem]) {
        guard let provider = item.first?.row.provider else { return }
        swapProviderSelectAction?(provider)
    }
}
