// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemPerpetualBalanceHeader
import struct Gemstone.GemSimulationValue
import struct Gemstone.GemWalletHomeViewState
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct ValueHeader {
    public let assetImage: AssetImage?
    public let title: String
    public let subtitle: String?
    public let subtitleColor: Color
    public let subtitleImage: Image?
    public let buttons: [HeaderButton]
    public let isWatchWallet: Bool

    public init(
        assetImage: AssetImage? = nil,
        title: String,
        subtitle: String? = nil,
        subtitleColor: Color = Colors.gray,
        subtitleImage: Image? = nil,
        buttons: [HeaderButton] = [],
        isWatchWallet: Bool = false,
    ) {
        self.assetImage = assetImage
        self.title = title
        self.subtitle = subtitle
        self.subtitleColor = subtitleColor
        self.subtitleImage = subtitleImage
        self.buttons = buttons
        self.isWatchWallet = isWatchWallet
    }

    public static func placeholder(assetImage: AssetImage) -> ValueHeader {
        ValueHeader(assetImage: assetImage, title: "")
    }
}

public extension GemWalletHomeViewState {
    var valueHeader: ValueHeader {
        ValueHeader(
            title: total.text(),
            subtitle: pnl?.text,
            subtitleColor: pnlTone.color,
            subtitleImage: Image(systemName: SystemImage.chartLineUptrendXyaxis),
            buttons: headerActions.headerButtons,
            isWatchWallet: headerActions.isWatchOnly,
        )
    }
}

public extension GemSimulationValue {
    var valueHeader: ValueHeader {
        ValueHeader(
            assetImage: AssetImage(icon: icon),
            title: value.title(symbol: asset.symbol, formatter: ValueFormatter(style: .full), decimals: Int(asset.decimals)),
        )
    }
}

public extension GemPerpetualBalanceHeader {
    var valueHeader: ValueHeader {
        ValueHeader(
            title: total.text(),
            subtitle: Localized.Wallet.availableBalance(available.text()),
            buttons: actions.headerButtons,
            isWatchWallet: actions.isWatchOnly,
        )
    }
}

extension AmountDisplay {
    var valueHeader: ValueHeader {
        ValueHeader(assetImage: assetImage, title: amount.text, subtitle: fiat?.text)
    }
}
