// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemEmptyStateImage
import enum Gemstone.GemPriceAlertToggle
import enum Gemstone.GemFiatTransactionBadge
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemTransactionStateTone
import enum Gemstone.LinkType
import struct Gemstone.GemPriceAlertRow
import enum Gemstone.GemValueTone
import enum Gemstone.PriceAlertDirection
import Components
import Primitives
import Style
import SwiftUI

extension PriceAlertDirection {
    public var color: Color {
        switch self {
        case .up: Colors.green
        case .down: Colors.red
        }
    }
}

extension GemEmptyStateImage {
    public var image: Image {
        switch self {
        case .nfts: Images.EmptyContent.nft
        case .priceAlerts: Images.EmptyContent.priceAlerts
        case .contacts: Images.EmptyContent.contacts
        case .activity, .wallet: Images.EmptyContent.activity
        case .stake: Images.EmptyContent.stake
        case .walletConnect: Images.EmptyContent.walletConnect
        case .notifications: Images.System.bell
        case .search: Images.EmptyContent.search
        }
    }
}

extension GemPriceAlertToggle {
    public var image: Image {
        switch self {
        case .enabled: Image(systemName: SystemImage.bellFill)
        case .disabled: Image(systemName: SystemImage.bell)
        }
    }
}

extension GemValueTone {
    public var color: Color {
        switch self {
        case .plain: Colors.black
        case .neutral: Colors.gray
        case .positive: Colors.green
        case .negative: Colors.red
        }
    }
}

extension GemPriceAlertRow {
    public var directionColor: Color {
        direction?.color ?? Colors.gray
    }
}

extension GemHeaderButtonKind {
    public var image: Image {
        switch self {
        case .send: Images.System.paperplane
        case .receive: Images.System.qrCode
        case .buy: Images.System.dollar
        case .swap: Images.System.arrowSwap
        case .deposit: Images.Actions.buy
        case .withdraw: Images.Actions.send
        case .more: Images.Actions.more
        }
    }
}

extension LinkType {
    public var image: Image {
        switch self {
        case .x: Images.Social.x
        case .discord: Images.Social.discord
        case .reddit: Images.Social.reddit
        case .telegram: Images.Social.telegram
        case .gitHub: Images.Social.github
        case .youTube: Images.Social.youtube
        case .facebook: Images.Social.facebook
        case .website: Images.Social.website
        case .coingecko: Images.Social.coingecko
        case .coinMarketCap: Images.Social.coinmarketcap
        case .openSea: Images.Social.opensea
        case .instagram: Images.Social.instagram
        case .magicEden: Images.Social.magiceden
        case .tikTok: Images.Social.tiktok
        }
    }
}

extension GemTransactionStateTone {
    public var image: Image {
        switch self {
        case .pending: Images.Transaction.State.pending
        case .success: Images.Transaction.State.success
        case .error, .refunded: Images.Transaction.State.error
        }
    }

    public var color: Color {
        switch self {
        case .success: Colors.green
        case .pending, .refunded: Colors.orange
        case .error: Colors.red
        }
    }
}

extension Primitives.PerpetualDirection {
    public var color: Color {
        switch self {
        case .long: Colors.green
        case .short: Colors.red
        }
    }
}

extension VerificationStatus {
    public var statusStyle: TextStyle {
        switch self {
        case .verified: .calloutSecondary
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
    }

    public var statusAssetImage: AssetImage {
        switch self {
        case .verified: AssetImage()
        case .unverified: AssetImage(placeholder: Images.TokenStatus.warning)
        case .suspicious: AssetImage(placeholder: Images.TokenStatus.risk)
        }
    }
}

extension GemFiatTransactionBadge {
    public var color: Color {
        switch self {
        case .pending: Colors.orange
        case .failed: Colors.red
        }
    }

    public var textStyle: TextStyle {
        TextStyle(font: Font.system(.footnote, weight: .medium), color: color, background: color.opacity(.light))
    }
}
