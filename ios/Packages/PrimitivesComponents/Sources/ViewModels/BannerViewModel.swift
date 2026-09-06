// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemBannerAmount
import struct Gemstone.GemBannerContent
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

struct BannerViewModel {
    enum BannerViewType {
        case list
        case banner
    }

    private let banner: Banner
    private let content: GemBannerContent

    init(banner: Banner, content: GemBannerContent) {
        self.banner = banner
        self.content = content
    }

    var image: AssetImage? {
        guard let icon = content.icon else {
            return .none
        }
        switch icon {
        case .moneyBag:
            return AssetImage(type: .emoji(Emoji.WalletAvatar.moneyBag.rawValue))
        case let .network(chain):
            return Primitives.Chain(rawValue: chain).map { AssetImage.image(ChainImage(chain: $0).image) }
        case .warning:
            return AssetImage.image(Images.System.exclamationmarkTriangle)
        case .suspicious:
            return AssetImage.image(Images.TokenStatus.risk)
        case .bitcoin:
            return AssetImage.image(Images.System.bitcoin)
        case .perpetuals:
            return AssetImage.image(Images.Perpetuals.perpetuals)
        }
    }

    var title: String? {
        guard let title = content.title else {
            return .none
        }
        switch title {
        case let .stake(assetName): return Localized.Banner.Stake.title(assetName)
        case .accountActivation: return Localized.Banner.AccountActivation.title
        case .warning: return Localized.Common.warning
        case .activateAsset: return Localized.Transfer.ActivateAsset.title
        case .suspiciousAsset: return Localized.Banner.AssetStatus.title
        case .onboarding: return Localized.Banner.Onboarding.title
        case .tradePerpetuals: return Localized.Banner.Perpetuals.title
        }
    }

    var description: String? {
        guard let description = content.description else {
            return .none
        }
        switch description {
        case let .stake(assetSymbol): return Localized.Banner.Stake.description(assetSymbol)
        case let .accountActivation(networkName, fee): return Localized.Banner.AccountActivation.description(networkName, formatted(fee))
        case let .multiSignatureBlocked(networkName): return Localized.Warnings.multiSignatureBlocked(networkName)
        case let .activateAsset(assetSymbol, networkName): return Localized.Banner.ActivateAsset.description(assetSymbol, networkName)
        case .suspiciousAsset: return Localized.Banner.AssetStatus.description
        case .onboarding: return Localized.Banner.Onboarding.description
        case .tradePerpetuals: return Localized.Banner.Perpetuals.description
        }
    }

    var canClose: Bool {
        banner.state != .alwaysActive
    }

    var imageSize: CGFloat {
        switch banner.event {
        case .stake,
             .accountActivation,
             .accountBlockedMultiSignature,
             .activateAsset,
             .suspiciousAsset,
             .tradePerpetuals: .image.asset
        case .onboarding: .image.medium
        }
    }

    var cornerRadius: CGFloat {
        switch banner.event {
        case .stake,
             .accountActivation,
             .activateAsset,
             .suspiciousAsset,
             .tradePerpetuals: 14
        case .accountBlockedMultiSignature,
             .onboarding: 0
        }
    }

    var action: BannerAction {
        BannerAction(banner: banner, type: .event(banner.event), url: url)
    }

    var closeAction: BannerAction {
        BannerAction(banner: banner, type: .closeBanner, url: nil)
    }

    var url: URL? {
        switch content.link {
        case let .docs(item): AppUrl.docs(item)
        case let .external(url): URL(string: url)
        case .none: .none
        }
    }

    var imageStyle: ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: image,
            imageSize: imageSize,
            cornerRadiusType: .custom(cornerRadius),
        )
    }

    var viewType: BannerViewType {
        switch banner.event {
        case .stake,
             .accountActivation,
             .accountBlockedMultiSignature,
             .activateAsset,
             .suspiciousAsset,
             .tradePerpetuals: .list
        case .onboarding: .banner
        }
    }

    var buttons: [BannerButtonViewModel] {
        switch banner.event {
        case .stake,
             .accountActivation,
             .accountBlockedMultiSignature,
             .activateAsset,
             .suspiciousAsset,
             .tradePerpetuals: []
        case .onboarding: [
                BannerButtonViewModel(button: .buy, banner: banner),
                BannerButtonViewModel(button: .receive, banner: banner),
            ]
        }
    }

    private func formatted(_ amount: GemBannerAmount) -> String {
        ValueFormatter(style: .auto)
            .string(amount.value, decimals: amount.decimals.asInt, currency: amount.symbol)
    }
}

extension BannerViewModel: Identifiable {
    var id: String {
        banner.id
    }
}
