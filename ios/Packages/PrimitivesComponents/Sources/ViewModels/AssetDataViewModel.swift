// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import class Gemstone.CryptoFiatConverter
import enum Gemstone.GemCurrencyStyle
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct AssetDataViewModel: Sendable {
    private let assetData: AssetData
    private let balanceViewModel: BalanceViewModel

    public let priceViewModel: PriceViewModel
    public let currency: Currency

    public init(
        assetData: AssetData,
        formatter: ValueFormatter,
        currency: Currency,
        currencyFormatterType: GemCurrencyStyle = .currency,
    ) {
        self.assetData = assetData
        priceViewModel = PriceViewModel(
            price: assetData.price,
            currencyCode: currency.rawValue,
            currencyFormatterType: currencyFormatterType,
        )
        balanceViewModel = BalanceViewModel(
            asset: assetData.asset,
            balance: assetData.balance,
            formatter: formatter,
        )
        self.currency = currency
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

    public var fiatBalanceText: String {
        guard balanceViewModel.balanceAmount > 0 else { return .empty }
        return priceViewModel.fiatValueText(value: balanceViewModel.total, decimals: asset.decimals.asInt) ?? .empty
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

    public var assetAddress: AssetAddress {
        assetData.assetAddress
    }
}
