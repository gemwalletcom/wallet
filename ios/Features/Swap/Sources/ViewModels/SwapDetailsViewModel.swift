// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.GemSwapDetailRow
import struct Gemstone.GemSwapQuoteSummary
import struct Gemstone.GemSwapRate
import struct Gemstone.SwapperQuote
import struct Gemstone.SwapPriceImpact
import struct Gemstone.SwapProviderData
import func Gemstone.swapProviderRow
import struct Gemstone.SwapQuote
import GemstonePrimitives
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
    private let providerData: SwapProviderData
    private let summary: GemSwapQuoteSummary
    private let selectedQuote: Gemstone.SwapQuote
    private let slippagePercent: Double?
    private let rate: GemSwapRate?
    private var isRateInverse = false
    private let priceViewModel: PriceViewModel
    private let currency: String
    let allowSelectProvider: Bool
    private let swapPriceImpact: SwapPriceImpact?
    private let minReceiveValue: BigInt
    private let etaSeconds: UInt32?
    private let swapProviderSelectAction: ((SwapperQuote) -> Void)?

    public init(
        state: StateViewType<[SwapProviderItem]> = .data([]),
        fromAssetPrice: AssetPriceValue,
        toAssetPrice: AssetPriceValue,
        summary: GemSwapQuoteSummary,
        slippagePercent: Double?,
        currency: String,
        allowSelectProvider: Bool = true,
        swapPriceImpact: SwapPriceImpact?,
        swapProviderSelectAction: ((SwapperQuote) -> Void)? = nil,
    ) {
        self.state = state
        self.fromAssetPrice = fromAssetPrice
        self.toAssetPrice = toAssetPrice
        providerData = summary.quote.providerData
        self.summary = summary
        selectedQuote = summary.quote
        self.slippagePercent = slippagePercent
        rate = summary.rate
        priceViewModel = PriceViewModel(price: toAssetPrice.price, currencyCode: currency)
        self.currency = currency
        self.allowSelectProvider = allowSelectProvider
        self.swapPriceImpact = swapPriceImpact
        minReceiveValue = BigInt(summary.minReceiveValue)
        etaSeconds = summary.quote.etaInSeconds
        self.swapProviderSelectAction = swapProviderSelectAction
    }

    var detailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    var providerTitle: String {
        GemSwapDetailRow.provider.title
    }

    var detailRowList: [SwapDetailRow] {
        detailRows.map { row in
            switch row {
            case .provider: .provider
            case .rate: .rate
            case .estimatedTime: .estimatedTime
            case .priceImpact: .priceImpact
            case .minimumReceive: .minimumReceive
            case .slippage: .slippage
            }
        }
    }

    var detailRows: [GemSwapDetailRow] {
        summary.rows(showsPriceImpact: shouldShowPriceImpactInDetails)
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

    // MARK: - Estimation

    var swapEstimationField: ListItemField? {
        guard let etaSeconds else { return nil }
        let estimationTime = EstimatedConfirmationFormatter().string(seconds: etaSeconds)
        guard estimationTime.isEmpty == false else { return nil }
        return ListItemField(title: GemSwapDetailRow.estimatedTime.title, value: estimationTime)
    }

    // MARK: - Rate

    var rateTitle: String {
        GemSwapDetailRow.rate.title
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
        let value = slippagePercent.map { percentSignLessFormatter.string($0) } ?? Localized.Swap.slippageAuto
        return ListItemField(title: GemSwapDetailRow.slippage.title, value: value)
    }

    // MARK: - Min receive

    var minReceiveField: ListItemField {
        ListItemField(
            title: GemSwapDetailRow.minimumReceive.title,
            value: valueFormatter.string(minReceiveValue, asset: toAssetPrice.asset),
        )
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
