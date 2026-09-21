// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerRow
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

public struct BannerViewModel {
    enum BannerViewType {
        case list
        case banner
    }

    private let banner: Banner
    private let content: GemBannerContent

    public init(row: GemBannerRow) {
        banner = row.banner.toPrimitives()
        content = row.content
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
        content.description?.text
    }

    var canClose: Bool {
        content.canClose
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

    var action: BannerAction? {
        content.destination.map { BannerAction(banner: banner, type: .destination($0)) }
    }

    var closeAction: BannerAction {
        BannerAction(banner: banner, type: .closeBanner)
    }

    var imageStyle: ListItemImageStyle? {
        ListItemImageStyle(
            assetImage: image,
            imageSize: imageSize,
            cornerRadiusType: .custom(cornerRadius),
        )
    }

    var viewType: BannerViewType {
        switch content.style {
        case .list: .list
        case .welcome: .banner
        }
    }

    var buttons: [BannerButtonViewModel] {
        content.buttons.map { BannerButtonViewModel(button: $0.button, banner: banner) }
    }
}

extension BannerViewModel: Identifiable {
    public var id: String {
        banner.id
    }
}
