// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import Style

struct NodeStatusStateViewModel {
    let nodeStatus: NodeStatusState

    func latestBlockText(title: String, formatter: ValueFormatter) -> String {
        let value = switch nodeStatus {
        case let .result(nodeStatus): formatter.string(nodeStatus.latestBlockNumber, decimals: 0)
        case .error, .none: "-"
        }
        return "\(title): \(value)"
    }

    var latencyText: String? {
        statusTag.text
    }

    var titleTagType: TitleTagType {
        statusTag.type
    }

    var titleTagStyle: TextStyle {
        statusTag.style
    }

    private var statusTag: LatencyStatusViewModel {
        LatencyStatusViewModel(nodeStatus: nodeStatus)
    }
}
