// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemNodeStatusState
import GemstonePrimitives
import Localization
import Primitives
import Style
import SwiftUI

struct LatencyStatusViewModel {
    enum State {
        case latency(Latency)
        case error
        case loading
    }

    let state: State

    var text: String? {
        switch state {
        case let .latency(latency): LatencyViewModel(latency: latency).title
        case .error: Localized.Errors.error
        case .loading: ""
        }
    }

    var type: TitleTagType {
        switch state {
        case .latency, .error: .none
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
        switch state {
        case let .latency(latency): LatencyViewModel(latency: latency).color
        case .error: Colors.red
        case .loading: Colors.gray
        }
    }

    private var background: Color {
        switch state {
        case let .latency(latency): LatencyViewModel(latency: latency).background
        case .error: Colors.red.opacity(.light)
        case .loading: .clear
        }
    }
}

extension LatencyStatusViewModel {
    init(serviceStatus: ServiceStatusState) {
        switch serviceStatus {
        case let .result(latency): self.init(state: .latency(latency))
        case .error: self.init(state: .error)
        case .loading: self.init(state: .loading)
        }
    }

    init(nodeStatus: GemNodeStatusState) {
        switch nodeStatus {
        case let .result(_, latency): self.init(state: .latency(latency.map()))
        case .error: self.init(state: .error)
        case .loading: self.init(state: .loading)
        }
    }
}
