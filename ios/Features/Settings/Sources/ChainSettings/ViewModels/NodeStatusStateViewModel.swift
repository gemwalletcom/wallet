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
        let value = nodeStatus.latestBlock().map { formatter.string(BigInt($0), decimals: 0) } ?? "-"
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
        LatencyStatusViewModel(status: nodeStatus.latencyStatus())
    }
}
