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
    public let assetData: AssetData

    public let priceViewModel: PriceViewModel
    public let currency: Currency

    public init(
        assetData: AssetData,
        currency: Currency,
        currencyFormatterType: GemCurrencyStyle = .currency,
    ) {
        self.assetData = assetData
        priceViewModel = PriceViewModel(
            price: assetData.price,
            currencyCode: currency.rawValue,
            currencyFormatterType: currencyFormatterType,
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
