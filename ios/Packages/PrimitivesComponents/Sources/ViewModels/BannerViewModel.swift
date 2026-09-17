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
        content.icon?.image
    }

    var listItem: ListItemModel {
        ListItemModel(title: title, titleExtra: description, imageStyle: imageStyle)
    }

    var title: String? {
        content.title?.text
    }

    var description: String? {
        content.description?.text(amount: formatted)
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
