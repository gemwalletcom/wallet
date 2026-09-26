// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import enum Gemstone.GemAcquireOption
import enum Gemstone.GemAmountKeyboard
import enum Gemstone.GemAmountSymbolPlacement
import struct Gemstone.GemAssetIcon
import struct Gemstone.GemAssetText
import struct Gemstone.GemAvatar
import enum Gemstone.GemBannerButton
import enum Gemstone.GemBannerIcon
import enum Gemstone.GemButtonState
import struct Gemstone.GemChainRow
import enum Gemstone.GemCollectibleAction
import enum Gemstone.GemContactAvatarImage
import enum Gemstone.GemEmptyStateImage
import enum Gemstone.GemFiatTransactionBadge
import enum Gemstone.GemHeaderActions
import struct Gemstone.GemHeaderAmount
import struct Gemstone.GemHeaderButton
import enum Gemstone.GemHeaderButtonKind
import enum Gemstone.GemInfoImage
import enum Gemstone.GemKeystoreAuthentication
import enum Gemstone.GemLatencyStatus
import enum Gemstone.GemListRowIcon
import enum Gemstone.GemLoadState
import enum Gemstone.GemNameIndicator
import enum Gemstone.GemNodeSyncState
import enum Gemstone.GemNoticeKind
import enum Gemstone.GemPerpetualChartLineKind
import enum Gemstone.GemPriceAlertToggle
import enum Gemstone.GemRecipientSectionKind
import enum Gemstone.GemSecretWarning
import struct Gemstone.GemSocialLink
import enum Gemstone.GemSwapProgressMarker
import struct Gemstone.GemSwapProgressState
import enum Gemstone.GemSwapProgressStep
import enum Gemstone.GemTransactionRowValue
import enum Gemstone.GemTransactionStateTone
import enum Gemstone.GemValueTone
import enum Gemstone.GemVerificationLevel
import enum Gemstone.GemWalletPlaceholder
import struct Gemstone.GemWalletRow
import enum Gemstone.GemWalletSectionKind
import enum Gemstone.LinkType
import enum Gemstone.YieldProvider
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

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

    @MainActor
    var buttonStyle: ColorButtonStyle {
        switch self {
        case .positive: .green()
        case .negative: .red()
        case .plain, .neutral, .warning: .blue()
        }
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

public extension GemNameIndicator {
    var image: Image? {
        switch self {
        case .loading: nil
        case .error: Images.NameResolve.error
        case .success: Images.NameResolve.success
        }
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

    var imageSize: CGFloat {
        switch self {
        case .bitcoin: .image.medium
        case .moneyBag, .network, .warning, .suspicious, .perpetuals: .image.asset
        }
    }

    var cornerRadius: CGFloat {
        switch self {
        case .warning, .bitcoin: 0
        case .moneyBag, .network, .suspicious, .perpetuals: 14
        }
    }

    var imageStyle: ListItemImageStyle? {
        ListItemImageStyle(assetImage: image, imageSize: imageSize, cornerRadiusType: .custom(cornerRadius))
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
        case .priceAlerts: AssetImage.image(Images.Settings.priceAlerts)
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
        case .none, .appLogo, .wallets, .security, .notifications, .priceAlerts, .preferences, .walletConnect, .support, .rewards, .aboutUs, .developer, .currency, .language, .appearance, .networks, .contacts, .perpetuals:
            .settings(assetImage: assetImage)
        case .pin, .unpin, .addToWallet:
            .list(assetImage: assetImage)
        }
    }
}

public extension GemPerpetualChartLineKind {
    var color: Color {
        switch self {
        case .takeProfit: Colors.green
        case .stopLoss: Colors.orange
        case .entry: Colors.gray
        case .liquidation: Colors.red
        }
    }
}

public extension GemNodeSyncState {
    var symbol: String {
        switch self {
        case .inSync: Emoji.checkmark
        case .outOfSync: Emoji.reject
        }
    }
}

public extension GemSwapProgressMarker {
    var image: Image? {
        switch self {
        case .check: Images.System.checkmark
        case .dots: Images.System.ellipsis
        case .cross: Images.System.xmark
        case .swap: Images.System.arrowSwap
        case .spinner: nil
        }
    }
}

public extension GemRecipientSectionKind {
    var image: Image {
        switch self {
        case .pinned: Images.System.pin
        case .contacts: Images.System.person
        case .wallets: Images.System.wallet
        case .viewWallets: Images.System.eye
        }
    }
}

public extension GemVerificationLevel {
    var image: Image {
        switch self {
        case .verified: Images.Transaction.State.success
        case .unverified: Images.TokenStatus.warning
        case .suspicious: Images.TokenStatus.risk
        }
    }

    var textStyle: TextStyle {
        switch self {
        case .verified: TextStyle(font: .callout, color: Colors.green)
        case .unverified: TextStyle(font: .callout, color: Colors.orange)
        case .suspicious: TextStyle(font: .callout, color: Colors.red)
        }
    }
}

public extension GemInfoImage {
    var sheetImage: InfoSheetImage {
        switch self {
        case .logo: .image(Images.Logo.logo)
        case .networkFee: .image(Images.Info.networkFee)
        case .watchWallet: .image(Images.Wallets.watch)
        case let .assetStatus(status): .assetImage(status.toPrimitives().statusAssetImage)
        case let .swapProvider(provider): .assetImage(AssetImage(placeholder: provider.toPrimitives().image))
        case let .asset(icon): .assetImage(AssetImage(icon: icon))
        case let .transactionState(icon, tone): .assetImage(AssetImage(icon: icon).badged(tone.image))
        }
    }
}

private extension AssetImage {
    func badged(_ badge: Image) -> AssetImage {
        AssetImage(imageURL: imageURL, placeholder: placeholder, chainPlaceholder: badge)
    }
}

public extension GemSwapProgressStep {
    var color: Color {
        switch self {
        case .completed: Colors.green
        case .pending: Colors.blue
        case .waiting: Colors.gray
        case .failed, .reverted: Colors.red
        case .refunded: Colors.orange
        }
    }

    var background: Color {
        color.opacity(.light)
    }

    var lineColor: Color {
        switch self {
        case .completed: Colors.green
        case .pending, .waiting, .failed, .reverted, .refunded: Colors.gray.opacity(.medium)
        }
    }
}

public extension GemSwapProgressState {
    var color: Color {
        step.color
    }

    var background: Color {
        step.background
    }

    var lineColor: Color {
        step.lineColor
    }

    var markerBackground: Color {
        switch marker {
        case .check, .cross, .swap: background
        case .spinner, .dots: .clear
        }
    }
}

public extension GemContactAvatarImage {
    var assetImage: AssetImage {
        switch self {
        case let .initials(text): AssetImage(type: .text(text))
        case .placeholder: .image(Images.System.personCircleFill)
        case let .image(imageUrl, initials): AssetImage(type: .text(initials), imageURL: ImageSource(imageUrl).url)
        case let .emoji(emoji): AssetImage(type: .emoji(emoji))
        }
    }

    var style: AssetImageView.Style? {
        switch self {
        case .placeholder: AssetImageView.Style(foregroundColor: Colors.grayLightFaded)
        case .initials, .image, .emoji: nil
        }
    }
}

public extension GemWalletPlaceholder {
    var image: Image {
        switch self {
        case .multicoin: Images.Logo.logo
        case let .chain(chain): ChainImage(chain: Primitives.Chain(core: chain)).image
        }
    }
}

public extension GemWalletRow {
    var avatarImage: AssetImage {
        AssetImage(
            type: .text(name),
            imageURL: imageUrl.map { ImageSource($0).url },
            placeholder: placeholder.image,
            chainPlaceholder: showsWatchBadge ? Images.Wallets.watch : nil,
        )
    }
}

public extension GemSecretWarning {
    var calloutViewStyle: CalloutViewStyle {
        switch self {
        case .doNotShare: .secretDataWarning()
        case .saveSafely: .header(title: Localized.SecretPhrase.savePhraseSafely)
        }
    }
}

public extension GemWalletSectionKind {
    var image: Image? {
        switch self {
        case .pinned: Images.System.pin
        case .wallets: nil
        }
    }
}

public extension FiatProviderName {
    var image: Image {
        switch self {
        case .moonPay: Images.Fiat.moonpay
        case .transak: Images.Fiat.transak
        case .banxa: Images.Fiat.banxa
        case .mercuryo: Images.Fiat.mercuryo
        case .paybis: Images.Fiat.paybis
        case .flashnet: Images.Fiat.cashapp
        }
    }
}

public extension GemAvatar {
    var assetImage: AssetImage {
        AssetImage(type: .text(initials), imageURL: imageUrl.map { ImageSource($0).url })
    }
}

public extension GemButtonState {
    var state: ButtonState {
        switch self {
        case .disabled: .disabled
        case .loading: .loading(showProgress: true)
        case .enabled: .normal
        }
    }
}

public extension GemAmountSymbolPlacement {
    var position: CurrencyTextField.CurrencyPosition {
        switch self {
        case .leading: .leading
        case .trailing: .trailing
        }
    }
}

public extension GemAmountKeyboard {
    var keyboardType: UIKeyboardType {
        switch self {
        case .decimal: .decimalPad
        case .whole: .numberPad
        }
    }
}

public extension GemAcquireOption {
    var image: Image {
        switch self {
        case .buy: Images.System.plus
        case .swap: Images.System.arrowSwap
        case .receive: Image(systemName: "arrow.down")
        }
    }

    var color: Color {
        switch self {
        case .buy: Colors.blue
        case .swap: Colors.green
        case .receive: Color.purple
        }
    }
}

public extension GemKeystoreAuthentication {
    var image: Image? {
        switch self {
        case .biometrics: Image(systemName: SystemImage.faceid)
        case .passcode: Image(systemName: SystemImage.lock)
        case .none: nil
        }
    }
}

public extension GemHeaderActions {
    var isWatchOnly: Bool {
        self == .watchOnly
    }

    var headerButtons: [GemHeaderButton] {
        switch self {
        case .watchOnly: []
        case let .buttons(buttons): buttons
        }
    }
}

public extension GemLatencyStatus {
    func listItem(title: String, titleExtra: String?) -> ListItemModel {
        let color = tone().color
        let badge: (text: String, type: TitleTagType, background: Color) = switch self {
        case let .result(latency): (Localized.Common.latencyInMs(Int(latency.value)), .none, color.opacity(.light))
        case .error: (Localized.Errors.error, .none, color.opacity(.light))
        case .loading: ("", .progressView(scale: 1.24), .clear)
        }
        return ListItemModel(
            title: title,
            titleTag: badge.text,
            titleTagStyle: TextStyle(font: .footnote.weight(.medium), color: color, background: badge.background),
            titleTagType: badge.type,
            titleExtra: titleExtra,
        )
    }
}

public extension GemLoadState {
    func stateViewType<T>(_ value: T?) -> StateViewType<T> {
        switch self {
        case .noData: .noData
        case .loading: value.map { .data($0) } ?? .loading
        case .data: value.map { .data($0) } ?? .noData
        case let .error(error): .error(error)
        }
    }

    func stateViewType<T>(_ values: [T]) -> StateViewType<[T]> {
        stateViewType(values.isEmpty ? nil : values)
    }
}

extension GemSocialLink {
    var listItem: ListItemModel {
        ListItemModel(title: linkType.title, subtitle: host, imageStyle: .settings(assetImage: .image(linkType.image)))
    }

    var deepLink: URL? {
        guard let path = url.asURL?.path().trimmingPrefix("/") else { return nil }

        return switch linkType {
        case .telegram: URL(string: "tg://resolve?domain=\(path)")
        case .x: URL(string: "twitter://user?screen_name=\(path)")
        case .youTube: URL(string: "youtube://www.youtube.com/\(path)")
        case .discord: URL(string: "https://discord.gg/\(path)")
        case .gitHub: URL(string: "https://github.com/\(path)")
        case .reddit, .facebook, .website, .coingecko, .openSea, .instagram, .magicEden, .coinMarketCap, .tikTok:
            nil
        }
    }
}

public extension GemHeaderAmount {
    var swapAmountField: SwapAmountField {
        SwapAmountField(
            assetId: asset.toPrimitives().id,
            assetImage: AssetImage(icon: icon),
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
            TextValue(
                text: asset.symbol,
                style: TextStyle(font: .body, color: Colors.black, fontWeight: .semibold),
                lineLimit: 1,
            )
        case let .number(number):
            TextValue(text: number.text(), style: textStyle)
        }
    }
}

public extension SwapProvider {
    var image: Image {
        switch self {
        case .uniswapV3, .uniswapV4: Images.SwapProviders.uniswap
        case .jupiter: Images.SwapProviders.jupiter
        case .pancakeswapV3: Images.SwapProviders.pancakeswap
        case .thorchain: Images.SwapProviders.thorchain
        case .mayachain: Images.SwapProviders.mayachain
        case .across: Images.SwapProviders.across
        case .oku: Images.SwapProviders.oku
        case .wagmi: Images.SwapProviders.wagmi
        case .cetusClmm: Images.SwapProviders.cetus
        case .stonfiV2: Images.SwapProviders.stonfi
        case .mayan: Images.SwapProviders.mayan
        case .chainflip: Images.SwapProviders.chainflip
        case .relay: Images.SwapProviders.relay
        case .aerodrome: Images.SwapProviders.aerodrome
        case .hyperliquid: Images.SwapProviders.hyperliquid
        case .nearIntents: Images.SwapProviders.nearIntents
        case .orca: Images.SwapProviders.orca
        case .panora: Images.SwapProviders.panora
        case .okx: Images.SwapProviders.okx
        case .squid: Images.SwapProviders.squid
        case .swapsXyz: Images.SwapProviders.swapsXyz
        }
    }
}

public extension YieldProvider {
    var image: Image {
        switch self {
        case .yo: Images.EarnProviders.yo
        }
    }
}

public extension GemBannerButton {
    @MainActor
    var style: ColorButtonStyle {
        switch self {
        case .buy: .blue(paddingVertical: .small)
        case .receive: .empty(paddingVertical: .small)
        }
    }
}

public extension GemCollectibleAction {
    var systemImage: String? {
        switch self {
        case .saveImage: SystemImage.gallery
        case .setAvatar: SystemImage.emoji
        case .refresh: SystemImage.refresh
        case .report: nil
        }
    }
}

public extension AssetImage {
    init(icon: GemAssetIcon) {
        let (imageURL, placeholder): (URL?, Image?) = switch icon.image {
        case let .local(chain): (.none, ChainImage(chain: Chain(core: chain)).image)
        case let .localToken(token): (.none, TokenImage(token: token).image)
        case let .remote(url): (URL(string: url), .none)
        }
        self.init(
            type: .text(icon.placeholder ?? .empty),
            imageURL: imageURL,
            placeholder: placeholder,
            chainPlaceholder: icon.badge.map { ChainImage(chain: Chain(core: $0)).image },
        )
    }
}

extension GemAssetText: AssetPreviewable {
    public var name: String {
        asset.name
    }

    public var assetImage: AssetImage {
        AssetImage(icon: icon)
    }
}

public extension GemChainRow {
    var listItem: ListItemModel {
        let standard = standard?.text
        return ListItemModel(
            title: title,
            titleStyle: standard == nil ? .body : .body.weight(.medium),
            titleExtra: standard,
            titleStyleExtra: .calloutSecondary,
            imageStyle: .asset(assetImage: AssetImage(icon: icon)),
        )
    }
}
