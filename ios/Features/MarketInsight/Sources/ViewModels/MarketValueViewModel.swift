// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import InfoSheet
import PrimitivesComponents
import Style
import SwiftUI

struct MarketValueViewModel {
    enum Action {
        case none
        case explorer(ExplorerContextData)
        case info(InfoSheetType)
    }

    func listItem(infoAction: (() -> Void)? = nil) -> ListItemModel {
        ListItemModel(
            title: title,
            titleTag: titleTag,
            titleTagStyle: titleTagStyle ?? .body,
            titleExtra: titleExtra,
            subtitle: subtitle,
            subtitleExtra: subtitleExtra,
            subtitleStyleExtra: subtitleExtraStyle ?? .calloutSecondary,
            infoAction: infoAction,
        )
    }

    let title: String
    let titleExtra: String?
    let subtitle: String
    let subtitleExtra: String?
    let subtitleExtraStyle: TextStyle?
    let action: Action
    let titleTag: String?
    let titleTagStyle: TextStyle?

    init(
        title: String,
        titleExtra: String? = .none,
        subtitle: String,
        subtitleExtra: String? = .none,
        subtitleExtraStyle: TextStyle? = .none,
        action: Action = .none,
        titleTag: String? = .none,
        titleTagStyle: TextStyle? = .none,
    ) {
        self.title = title
        self.titleExtra = titleExtra
        self.subtitle = subtitle
        self.subtitleExtra = subtitleExtra
        self.subtitleExtraStyle = subtitleExtraStyle
        self.action = action
        self.titleTag = titleTag
        self.titleTagStyle = titleTagStyle
    }
}
