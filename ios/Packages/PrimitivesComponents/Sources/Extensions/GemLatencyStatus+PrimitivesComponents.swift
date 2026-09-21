// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemLatencyStatus
import Localization
import Style
import SwiftUI

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
