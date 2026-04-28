// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style

struct ConnectivityItemViewModel: Identifiable {
    private let endpoint: ConnectivityEndpoint
    private let statusState: ConnectivityStatusState

    init(
        endpoint: ConnectivityEndpoint,
        statusState: ConnectivityStatusState,
    ) {
        self.endpoint = endpoint
        self.statusState = statusState
    }

    var id: String {
        endpoint.id
    }

    var title: String {
        endpoint.title
    }

    var subtitle: String {
        endpoint.host
    }

    var titleTag: String? {
        statusTag.text
    }

    var titleTagType: TitleTagType {
        statusTag.type
    }

    var titleTagStyle: TextStyle {
        statusTag.style
    }

    private var statusTag: LatencyStatusViewModel {
        switch statusState {
        case let .result(latency): LatencyStatusViewModel(state: .latency(latency))
        case .error: LatencyStatusViewModel(state: .error)
        case .none: LatencyStatusViewModel(state: .loading)
        }
    }
}
