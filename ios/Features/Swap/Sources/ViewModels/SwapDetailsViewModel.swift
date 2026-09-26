// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemListRow
import struct Gemstone.GemProviderRow
import struct Gemstone.GemSwapDetails
import enum Gemstone.SwapProvider
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style

public struct SwapDetailsViewModel {
    let state: StateViewType<[GemProviderRow]>
    private let details: GemSwapDetails
    let allowSelectProvider: Bool
    private let swapProviderSelectAction: ((SwapProvider) -> Void)?

    public init(
        state: StateViewType<[GemProviderRow]> = .data([]),
        details: GemSwapDetails,
        allowSelectProvider: Bool = true,
        swapProviderSelectAction: ((SwapProvider) -> Void)? = nil,
    ) {
        self.state = state
        self.details = details
        self.allowSelectProvider = allowSelectProvider
        self.swapProviderSelectAction = swapProviderSelectAction
    }

    var detailsListItem: ListItemModel {
        ListItemModel(title: Localized.Common.details)
    }

    var providerTitle: String {
        Localized.Common.provider
    }

    var detailRows: [GemListRow] {
        details.rows
    }

    // MARK: - Provider

    var selectedProviderItem: GemProviderRow {
        details.provider
    }

    var swapProvidersViewModel: SwapProvidersViewModel {
        SwapProvidersViewModel(state: state.map { .plain($0) })
    }

    // MARK: - Rate

    var rateTitle: String {
        Localized.Buy.rate
    }

    func rateText(isInverse: Bool) -> String? {
        details.summary.rate.map { AssetRateViewModel(rate: $0).text(isInverse: isInverse) }
    }

    // MARK: - Price Impact

    var highImpactWarningTitle: String {
        Localized.Swap.PriceImpactWarning.title
    }

    var highImpactWarningDescription: String? {
        details.summary.priceImpactRow?.warning?.text
    }

    var shouldShowPriceImpactInDetails: Bool {
        details.summary.priceImpactRow?.showsInSummary == true
    }

    var priceImpactValue: String? {
        details.summary.priceImpactRow?.value.text()
    }

    var priceImpactStyle: TextStyle {
        TextStyle(font: .callout, color: details.summary.priceImpactRow?.value.tone.color ?? Colors.gray)
    }
}

// MARK: - Actions

extension SwapDetailsViewModel {
    func onFinishSwapProviderSelection(item: [GemProviderRow]) {
        guard case let .swap(provider) = item.first?.kind else { return }
        swapProviderSelectAction?(provider)
    }
}
