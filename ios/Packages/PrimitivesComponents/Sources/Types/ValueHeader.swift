// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemValueHeader
import enum Gemstone.GemValueHeaderIcon
import enum Gemstone.GemValueHeaderSubtitleIcon
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

public extension GemValueHeader {
    var valueHeader: ValueHeader {
        ValueHeader(
            assetImage: icon?.assetImage,
            title: title.text,
            subtitle: subtitle?.text.text,
            subtitleColor: subtitle?.tone.color ?? Colors.gray,
            subtitleImage: subtitleIcon?.image,
            buttons: actions?.headerButtons ?? [],
            isWatchWallet: actions?.isWatchOnly ?? false,
        )
    }
}

extension GemValueHeaderIcon {
    var assetImage: AssetImage {
        switch self {
        case let .asset(icon): AssetImage(icon: icon)
        case let .image(url, placeholder): AssetImage(type: .text(placeholder ?? .empty), imageURL: URL(string: url), placeholder: .none, chainPlaceholder: .none)
        }
    }
}

extension GemValueHeaderSubtitleIcon {
    var image: Image {
        switch self {
        case .chart: Image(systemName: SystemImage.chartLineUptrendXyaxis)
        }
    }
}

extension AmountDisplay {
    var valueHeader: ValueHeader {
        ValueHeader(assetImage: assetImage, title: amount.text, subtitle: fiat?.text)
    }
}
