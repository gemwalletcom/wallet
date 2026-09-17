// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemLatencyStatus
import struct Gemstone.GemServiceEndpoint
import Localization
import Style

struct ServiceStatusItemViewModel: Identifiable {
    private let endpoint: GemServiceEndpoint
    private let status: GemLatencyStatus

    init(
        endpoint: GemServiceEndpoint,
        status: GemLatencyStatus,
    ) {
        self.endpoint = endpoint
        self.status = status
    }

    var id: String { endpoint.url }
    var title: String { endpoint.title(name: endpoint.endpointType.name) }
    var subtitle: String { endpoint.host }
    var titleTag: String? { statusTag.text }
    var titleTagType: TitleTagType { statusTag.type }
    var titleTagStyle: TextStyle { statusTag.style }

    var listItem: ListItemModel {
        ListItemModel(title: title, titleTag: titleTag, titleTagStyle: titleTagStyle, titleTagType: titleTagType, titleExtra: subtitle)
    }

    private var statusTag: LatencyStatusViewModel {
        LatencyStatusViewModel(status: status)
    }
}
