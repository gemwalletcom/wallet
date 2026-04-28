// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Primitives
import Style

struct NodeStatusStateViewModel: Sendable {
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
        switch nodeStatus {
        case let .result(nodeStatus):
            if nodeStatus.latestBlockNumber > 0 {
                return LatencyStatusViewModel(state: .latency(nodeStatus.latency))
            }
            return LatencyStatusViewModel(state: .error)
        case .error:
            return LatencyStatusViewModel(state: .error)
        case .none:
            return LatencyStatusViewModel(state: .loading)
        }
    }
}
