// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemServiceEndpoint
import Localization
import Primitives
import Style

struct ServiceStatusItemViewModel: Identifiable {
    private let endpoint: GemServiceEndpoint
    private let statusState: ServiceStatusState

    init(
        endpoint: GemServiceEndpoint,
        statusState: ServiceStatusState,
    ) {
        self.endpoint = endpoint
        self.statusState = statusState
    }

    var id: String { endpoint.url }
    var title: String { "\(name) \(endpoint.flag)" }
    var subtitle: String { endpoint.host }
    var titleTag: String? { statusTag.text }
    var titleTagType: TitleTagType { statusTag.type }
    var titleTagStyle: TextStyle { statusTag.style }

    private var name: String {
        switch endpoint.endpointType {
        case .api: "API"
        case .gemNode: Localized.Nodes.gemWalletNode
        }
    }

    private var statusTag: LatencyStatusViewModel {
        switch statusState {
        case let .result(milliseconds): LatencyStatusViewModel(state: .latency(.from(duration: Double(milliseconds))))
        case .error: LatencyStatusViewModel(state: .error)
        case .loading: LatencyStatusViewModel(state: .loading)
        }
    }
}
