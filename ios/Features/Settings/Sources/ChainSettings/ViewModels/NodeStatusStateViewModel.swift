// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import enum Gemstone.GemNodeStatusState
import Primitives
import Style

struct NodeStatusStateViewModel {
    let nodeStatus: GemNodeStatusState

    func latestBlockText(title: String, formatter: ValueFormatter) -> String {
        let value = switch nodeStatus {
        case let .result(latestBlockNumber, _): formatter.string(BigInt(latestBlockNumber), decimals: 0)
        case .error, .loading: "-"
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
