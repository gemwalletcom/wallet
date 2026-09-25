// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemHeaderAmount
import enum Gemstone.GemTransactionHeader
import enum Gemstone.GemTransactionRowValue
import GemstonePrimitives
import Primitives
import Style

public extension GemHeaderAmount {
    var swapAmountField: SwapAmountField {
        let assetId = asset.toPrimitives().id
        return SwapAmountField(
            assetId: assetId,
            assetImage: AssetIdViewModel(assetId: assetId).assetImage,
            amount: amount.text(),
            fiatAmount: fiat?.text(),
        )
    }
}

public extension GemTransactionRowValue {
    func textValue(textStyle: TextStyle) -> TextValue? {
        switch self {
        case .none:
            nil
        case let .assetSymbol(asset):
            AmountDisplay.symbol(asset: asset.toPrimitives()).amount
        case let .number(number):
            TextValue(text: number.text(), style: textStyle)
        }
    }
}

public extension GemTransactionHeader {
    var headerType: TransactionHeaderType {
        switch self {
        case let .amount(amount):
            .amount(.numeric(NumericViewModel(header: amount)))
        case let .swap(from, to):
            .swap(from: from.swapAmountField, to: to.swapAmountField)
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
            .amount(.symbol(asset: asset.toPrimitives()))
        case let .assetImage(asset):
            .asset(image: AssetViewModel(asset: asset.toPrimitives()).assetImage)
        }
    }
}
