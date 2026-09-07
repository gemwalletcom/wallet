// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import enum Gemstone.GemAmountSign
import struct Gemstone.GemTransactionAmount
import enum Gemstone.GemTransactionHeader
import enum Gemstone.GemTransactionRowValue
import GemstonePrimitives
import Primitives
import Style

public extension GemTransactionAmount {
    func display(currency: String, formatter: ValueFormatter, textStyle: TextStyle? = nil) -> AmountDisplay {
        .numeric(
            data: AssetValuePrice(asset: asset.map(), value: BigInt(value), price: price.map { $0.map().mapToPrice() }),
            style: AmountDisplayStyle(sign: sign.displaySign, formatter: formatter, currencyCode: currency, textStyle: textStyle),
        )
    }

    func swapAmountField(currency: String) -> SwapAmountField {
        let display = display(currency: currency, formatter: .auto)
        let assetId = asset.map().id
        return SwapAmountField(
            assetId: assetId,
            assetImage: AssetIdViewModel(assetId: assetId).assetImage,
            amount: display.amount.text,
            fiatAmount: display.fiat?.text,
        )
    }
}

public extension GemAmountSign {
    var displaySign: AmountDisplaySign {
        switch self {
        case .incoming: .incoming
        case .outgoing: .outgoing
        case .none: .none
        }
    }
}

public extension GemTransactionRowValue {
    func textValue(currency: String, formatter: ValueFormatter, textStyle: TextStyle? = nil) -> TextValue? {
        switch self {
        case .none:
            nil
        case let .assetSymbol(asset):
            AmountDisplay.symbol(asset: asset.map()).amount
        case let .amount(amount):
            amount.display(currency: currency, formatter: formatter, textStyle: textStyle).amount
        case let .fiat(value):
            TextValue(
                text: CurrencyFormatter(type: .currency, currencyCode: Currency.usd.rawValue).string(value),
                style: TextStyle(font: .body, color: Colors.black, fontWeight: .medium),
            )
        case let .pnl(value):
            AmountDisplay.currency(value: value, currencyCode: Currency.usd.rawValue)
        }
    }
}

public extension GemTransactionHeader {
    func headerType(currency: String) -> TransactionHeaderType {
        switch self {
        case let .amount(amount, showsFiat):
            .amount(amount.display(currency: currency, formatter: .full).fiatVisibility(showsFiat))
        case let .swap(from, to):
            .swap(from: from.swapAmountField(currency: currency), to: to.swapAmountField(currency: currency))
        case let .nft(_, name, imageUrl):
            .nft(
                name: name,
                image: AssetImage(
                    type: .text("NFT"),
                    imageURL: URL(string: imageUrl),
                    placeholder: .none,
                    chainPlaceholder: .none,
                ),
            )
        case let .symbol(asset):
            .amount(.symbol(asset: asset.map()))
        case let .assetImage(asset):
            .asset(image: AssetViewModel(asset: asset.map()).assetImage)
        }
    }
}
