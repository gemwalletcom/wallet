// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemLatencyStatus
import Localization
import PrimitivesComponents
import Style
import SwiftUI

struct LatencyStatusViewModel {
    let status: GemLatencyStatus

    var text: String? {
        switch status {
        case let .result(latency): Localized.Common.latencyInMs(Int(latency.value))
        case .error: Localized.Errors.error
        case .loading: ""
        }
    }

    var type: TitleTagType {
        switch status {
        case .result, .error: .none
        case .loading: .progressView(scale: 1.24)
        }
    }

    var style: TextStyle {
        TextStyle(
            font: .footnote.weight(.medium),
            color: status.tone().color,
            background: background,
        )
    }

    private var background: Color {
        switch status {
        case .result, .error: status.tone().color.opacity(.light)
        case .loading: .clear
        }
    }
}
