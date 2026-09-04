// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style
import SwiftUI

public struct AssetDataViewModel: Sendable {
    private let assetData: AssetData
    private let balanceViewModel: BalanceViewModel

    public let priceViewModel: PriceViewModel
    public let currencyCode: String

    public init(
        assetData: AssetData,
        formatter: ValueFormatter,
        currencyCode: String,
        currencyFormatterType: CurrencyFormatterType = .currency,
    ) {
        self.assetData = assetData
        priceViewModel = PriceViewModel(
            price: assetData.price,
            currencyCode: currencyCode,
            currencyFormatterType: currencyFormatterType,
        )
        balanceViewModel = BalanceViewModel(
            asset: assetData.asset,
            balance: assetData.balance,
            formatter: formatter,
        )
        self.currencyCode = currencyCode
    }

    public var availableBalanceTitle: String {
        Localized.Asset.Balances.available
    }

    public var reservedBalanceTitle: String {
        Localized.Asset.Balances.reserved
    }

    public var pendingUnconfirmedBalanceTitle: String {
        Localized.Stake.pending
    }

    // asset

    public var assetImage: AssetImage {
        AssetIdViewModel(assetId: asset.id).assetImage
    }

    public var asset: Asset {
        assetData.asset
    }

    public var name: String {
        assetData.asset.name
    }

    public var symbol: String {
        assetData.asset.symbol
    }

    // price

    public var isPriceAvailable: Bool {
        priceViewModel.isPriceAvailable
    }

    public var priceAmountText: String {
        priceViewModel.priceAmountText
    }

    public var priceChangeText: String {
        priceViewModel.priceChangeText
    }

    public var priceChangeTextColor: Color {
        priceViewModel.priceChangeTextColor
    }

    // balance

    public var balanceText: String {
        balanceViewModel.balanceText
    }

    public var availableBalanceText: String {
        balanceViewModel.availableBalanceText
    }

    public var totalBalanceTextWithSymbol: String {
        balanceViewModel.totalBalanceTextWithSymbol
    }

    public var availableBalanceTextWithSymbol: String {
        balanceViewModel.availableBalanceTextWithSymbol
    }

    public func balanceTextWithSymbol(_ value: BigInt) -> String {
        balanceViewModel.balanceTextWithSymbol(value)
    }

    public var hasAvailableBalance: Bool {
        balanceViewModel.availableBalanceAmount > 0
    }

    public var balanceTextColor: Color {
        balanceViewModel.balanceTextColor
    }

    public var energyText: String {
        balanceViewModel.energyText
    }

    public var bandwidthText: String {
        balanceViewModel.bandwidthText
    }

    public var fiatBalanceText: String {
        guard
            let price = priceViewModel.price,
            balanceViewModel.balanceAmount > 0
        else {
            return .empty
        }
        let value = balanceViewModel.balanceAmount * price.price
        return CurrencyFormatter(
            type: .currency,
            currencyCode: currencyCode,
        ).string(value)
    }

    public var isEnabled: Bool {
        assetData.metadata.isBalanceEnabled
    }

    public var isBuyEnabled: Bool {
        assetData.metadata.isBuyEnabled
    }

    public var isSwapEnabled: Bool {
        assetData.metadata.isSwapEnabled
    }

    public var isStakeEnabled: Bool {
        assetData.metadata.isStakeEnabled
    }

    public var isActive: Bool {
        assetData.metadata.isActive
    }

    public var address: String {
        assetData.account.address
    }

    public func apr(for type: StakeProviderType) -> Double? {
        switch type {
        case .stake: assetData.metadata.stakingApr
        case .earn: assetData.metadata.earnApr
        }
    }

    public var isPriceAlertsEnabled: Bool {
        assetData.isPriceAlertsEnabled
    }

    public var assetAddress: AssetAddress {
        assetData.assetAddress
    }

    public var showResources: Bool {
        balanceViewModel.hasStakingResources
    }
}
