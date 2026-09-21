// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemBannerIcon
import enum Gemstone.GemEmptyStateImage
import enum Gemstone.GemFiatTransactionBadge
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemListRowIcon
import enum Gemstone.GemNoticeKind
import struct Gemstone.GemPriceAlertRow
import enum Gemstone.GemPriceAlertToggle
import enum Gemstone.GemTransactionStateTone
import enum Gemstone.GemValueTone
import enum Gemstone.LinkType
import enum Gemstone.PriceAlertDirection
import Primitives
import Style
import SwiftUI

public extension PriceAlertDirection {
    var color: Color {
        switch self {
        case .up: Colors.green
        case .down: Colors.red
        }
    }
}

public extension GemEmptyStateImage {
    var image: Image {
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

public extension GemPriceAlertToggle {
    var image: Image {
        switch self {
        case .enabled: Image(systemName: SystemImage.bellFill)
        case .disabled: Image(systemName: SystemImage.bell)
        }
    }
}

public extension GemValueTone {
    var color: Color {
        switch self {
        case .plain: Colors.black
        case .neutral: Colors.gray
        case .positive: Colors.green
        case .warning: Colors.orange
        case .negative: Colors.red
        }
    }

    var backgroundColor: Color {
        switch self {
        case .plain, .neutral: Colors.grayVeryLight
        case .positive: Colors.greenLight
        case .warning: Colors.orange.opacity(.light)
        case .negative: Colors.redLight
        }
    }
}

public extension GemPriceAlertRow {
    var directionColor: Color {
        direction?.color ?? Colors.gray
    }
}

public extension GemHeaderButtonKind {
    var image: Image {
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

public extension LinkType {
    var image: Image {
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

extension GemNoticeKind {
    var color: Color {
        switch self {
        case .error: Colors.red
        case .warning, .info: Colors.orange
        }
    }
}

public extension GemTransactionStateTone {
    var image: Image {
        switch self {
        case .pending: Images.Transaction.State.pending
        case .success: Images.Transaction.State.success
        case .error, .refunded: Images.Transaction.State.error
        }
    }

    var color: Color {
        switch self {
        case .success: Colors.green
        case .pending, .refunded: Colors.orange
        case .error: Colors.red
        }
    }
}

public extension Primitives.PerpetualDirection {
    var color: Color {
        switch self {
        case .long: Colors.green
        case .short: Colors.red
        }
    }
}

public extension VerificationStatus {
    var statusStyle: TextStyle {
        switch self {
        case .verified: .calloutSecondary
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
    }

    var statusAssetImage: AssetImage {
        switch self {
        case .verified: AssetImage()
        case .unverified: AssetImage(placeholder: Images.TokenStatus.warning)
        case .suspicious: AssetImage(placeholder: Images.TokenStatus.risk)
        }
    }
}

public extension GemFiatTransactionBadge {
    var color: Color {
        switch self {
        case .pending: Colors.orange
        case .failed: Colors.red
        }
    }

    var textStyle: TextStyle {
        TextStyle(font: Font.system(.footnote, weight: .medium), color: color, background: color.opacity(.light))
    }
}

public extension GemBannerIcon {
    var image: AssetImage? {
        switch self {
        case .moneyBag: AssetImage(type: .emoji(Emoji.WalletAvatar.moneyBag.rawValue))
        case let .network(chain): Primitives.Chain(rawValue: chain).map { AssetImage.image(ChainImage(chain: $0).image) }
        case .warning: AssetImage.image(Images.System.exclamationmarkTriangle)
        case .suspicious: AssetImage.image(Images.TokenStatus.risk)
        case .bitcoin: AssetImage.image(Images.System.bitcoin)
        case .perpetuals: AssetImage.image(Images.Perpetuals.perpetuals)
        }
    }
}

public extension GemListRowIcon {
    var assetImage: AssetImage? {
        switch self {
        case .none: nil
        case .appLogo: AssetImage.image(Images.Settings.gem)
        case .wallets: AssetImage.image(Images.Settings.wallets)
        case .security: AssetImage.image(Images.Settings.security)
        case .notifications: AssetImage.image(Images.Settings.notifications)
        case .preferences: AssetImage.image(Images.Settings.preferences)
        case .walletConnect: AssetImage.image(Images.Settings.walletConnect)
        case .support: AssetImage.image(Images.Settings.support)
        case .rewards: AssetImage.image(Images.Settings.gem)
        case .aboutUs: AssetImage.image(Images.Settings.aboutUs)
        case .developer: AssetImage.image(Images.Settings.developer)
        case .currency: AssetImage.image(Images.Settings.currency)
        case .language: AssetImage.image(Images.Settings.language)
        case .appearance: AssetImage.image(Images.Settings.appearance)
        case .networks: AssetImage.image(Images.Settings.networks)
        case .contacts: AssetImage.image(Images.Settings.contacts)
        case .perpetuals: AssetImage.image(Images.Settings.perpetuals)
        case .pin: AssetImage(placeholder: Image(systemName: SystemImage.pin))
        case .unpin: AssetImage(placeholder: Image(systemName: SystemImage.unpin))
        case .addToWallet: AssetImage(placeholder: Image(systemName: SystemImage.plusCircle))
        }
    }

    var imageStyle: ListItemImageStyle? {
        switch self {
        case .none, .appLogo, .wallets, .security, .notifications, .preferences, .walletConnect, .support, .rewards, .aboutUs, .developer, .currency, .language, .appearance, .networks, .contacts, .perpetuals:
            .settings(assetImage: assetImage)
        case .pin, .unpin, .addToWallet:
            .list(assetImage: assetImage)
        }
    }
}
