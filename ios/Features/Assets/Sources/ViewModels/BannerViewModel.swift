// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemBannerContent
import struct Gemstone.GemBannerRow
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct BannerViewModel {
    enum BannerViewType {
        case list
        case banner
    }

    public let id: String
    private let row: GemBannerRow

    public init(row: GemBannerRow) {
        id = row.key.identifier()
        self.row = row
    }

    private var content: GemBannerContent {
        row.content
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
        switch content.icon {
        case .bitcoin: .image.medium
        case .moneyBag, .network, .warning, .suspicious, .perpetuals, .none: .image.asset
        }
    }

    var cornerRadius: CGFloat {
        switch content.icon {
        case .warning, .bitcoin: 0
        case .moneyBag, .network, .suspicious, .perpetuals, .none: 14
        }
    }

    var action: BannerAction? {
        content.destination.map { BannerAction(key: row.key, type: .destination($0)) }
    }

    var closeAction: BannerAction {
        BannerAction(key: row.key, type: .closeBanner)
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
        content.buttons.map { BannerButtonViewModel(button: $0, key: row.key) }
    }
}

extension BannerViewModel: Identifiable {}
