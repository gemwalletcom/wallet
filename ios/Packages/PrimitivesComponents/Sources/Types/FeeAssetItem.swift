// Copyright (c). Gem Wallet. All rights reserved.

import GemstonePrimitives
import struct Gemstone.GemFeeAsset
import Components
import Formatters
import Primitives
import Style

public struct FeeAssetItem: Sendable {
    public let asset: Asset
    public let balance: Balance
    public let price: Price?
    public let currency: Currency
    public let isSelected: Bool

    public init(asset: Asset, balance: Balance, price: Price?, currency: Currency, isSelected: Bool) {
        self.asset = asset
        self.balance = balance
        self.price = price
        self.currency = currency
        self.isSelected = isSelected
    }
}

extension FeeAssetItem: SimpleListItemViewable {
    public var title: String { asset.symbol }
    public var titleExtra: String? { asset.name == title ? nil : asset.name }
    public var subtitle: String? { balanceModel.availableBalanceTextWithSymbol }
    public var subtitleExtra: String? { fiatBalanceText.isEmpty ? nil : fiatBalanceText }

    public var titleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleStyle: TextStyle {
        TextStyle(font: .callout, color: Colors.black, fontWeight: .semibold)
    }

    public var subtitleStyleExtra: TextStyle {
        TextStyle(font: .footnote, color: Colors.gray)
    }

    public var assetImage: AssetImage {
        let image = AssetViewModel(asset: asset).assetImage
        return AssetImage(
            type: image.type,
            imageURL: image.imageURL,
            placeholder: image.placeholder,
            chainPlaceholder: isSelected ? Images.Wallets.selected : nil,
        )
    }

    private var balanceModel: BalanceViewModel {
        BalanceViewModel(asset: asset, balance: balance, formatter: .short)
    }

    private var fiatBalanceText: String {
        guard let price, balanceModel.balanceAmount > 0 else { return .empty }
        return PriceViewModel(price: price, currencyCode: currency.rawValue).fiatAmountText(amount: balanceModel.balanceAmount)
    }
}

extension FeeAssetItem: Identifiable {
    public var id: AssetId { asset.id }
}

extension FeeAssetItem: Hashable {
    public static func == (lhs: FeeAssetItem, rhs: FeeAssetItem) -> Bool {
        lhs.id == rhs.id
    }

    public func hash(into hasher: inout Hasher) {
        id.hash(into: &hasher)
    }
}

extension FeeAssetItem {
    public func selected(_ isSelected: Bool) -> FeeAssetItem {
        FeeAssetItem(asset: asset, balance: balance, price: price, currency: currency, isSelected: isSelected)
    }
}

public extension GemFeeAsset {
    func feeAssetItem(currency: Currency) -> FeeAssetItem {
        let mapped = map()
        return FeeAssetItem(asset: mapped.asset, balance: mapped.balance, price: mapped.price, currency: currency, isSelected: false)
    }
}
