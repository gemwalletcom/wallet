import BigInt
import Components
import Formatters
import GemstonePrimitives
import Primitives
import Style
import SwiftUI

public struct NumericViewModel: Sendable, AmountDisplayable {
    public let data: AssetValuePrice
    public let style: AmountDisplayStyle

    public init(data: AssetValuePrice, style: AmountDisplayStyle) {
        self.data = data
        self.style = style
    }

    public var amount: TextValue {
        let number = style.sign.amount(
            value: data.value,
            decimals: UInt32(data.asset.decimals),
            symbol: data.asset.symbol,
            style: style.formatter.style,
        )
        let viewStyle = style.textStyle ?? TextStyle(
            font: .body,
            color: color,
            fontWeight: .medium,
        )
        return TextValue(
            text: number.text(locale: style.formatter.locale),
            style: viewStyle,
            lineLimit: 1,
        )
    }

    public var fiat: TextValue? {
        guard let text = PriceViewModel(price: data.price, currencyCode: style.currencyCode)
            .fiatValueText(value: data.value, decimals: data.asset.decimals.asInt)
        else { return nil }

        return TextValue(
            text: text,
            style: style.textStyle ?? TextStyle(font: .footnote, color: Colors.gray, fontWeight: .medium),
            lineLimit: 1,
        )
    }

    public var assetImage: AssetImage? {
        AssetViewModel(asset: data.asset).assetImage
    }

    private var color: Color {
        switch style.sign {
        case .incoming: Colors.green
        case .outgoing, .none: Colors.black
        }
    }
}
