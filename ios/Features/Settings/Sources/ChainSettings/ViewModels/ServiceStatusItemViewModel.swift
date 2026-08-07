// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style

struct ServiceStatusItemViewModel: Identifiable {
    private let endpoint: ServiceEndpoint
    private let statusState: ServiceStatusState

    init(
        endpoint: ServiceEndpoint,
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
        switch endpoint.type {
        case .api: "API"
        case .gemNode: Localized.Nodes.gemWalletNode
        }
    }

    private var statusTag: LatencyStatusViewModel {
        LatencyStatusViewModel(serviceStatus: statusState)
    }
}
