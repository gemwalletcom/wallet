// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemLatencyStatus
import Localization
import Style
import SwiftUI

struct LatencyStatusViewModel {
    let status: GemLatencyStatus

    var text: String? {
        switch status {
        case let .result(latency): LatencyViewModel(latency: latency).title
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
            color: color,
            background: background,
        )
    }

    private var color: Color {
        switch status {
        case let .result(latency): LatencyViewModel(latency: latency).color
        case .error: Colors.red
        case .loading: Colors.gray
        }
    }

    private var background: Color {
        switch status {
        case let .result(latency): LatencyViewModel(latency: latency).background
        case .error: Colors.red.opacity(.light)
        case .loading: .clear
        }
    }
}
