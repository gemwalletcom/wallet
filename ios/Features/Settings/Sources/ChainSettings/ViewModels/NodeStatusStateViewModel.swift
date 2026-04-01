// Copyright (c). Gem Wallet. All rights reserved.

import Formatters
import Localization
import Primitives
import Style
import SwiftUI

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
        switch nodeStatus {
        case let .result(nodeStatus):
            if nodeStatus.latestBlockNumber > 0 {
                return LatencyViewModel(latency: nodeStatus.latency).title
            }
            return Localized.Errors.error
        case .error:
            return Localized.Errors.error
        case .none:
            return ""
        }
    }

    var color: Color {
        switch nodeStatus {
        case .error: Colors.red
        case .none: Colors.gray
        case let .result(nodeStatus):
            nodeStatus.latestBlockNumber.isZero ? Colors.red : LatencyViewModel(latency: nodeStatus.latency).color
        }
    }

    var background: Color {
        switch nodeStatus {
        case .error: Colors.red.opacity(.light)
        case .none: .clear
        case let .result(nodeStatus):
            nodeStatus.latestBlockNumber.isZero ? Colors.red.opacity(.light) : LatencyViewModel(latency: nodeStatus.latency).background
        }
    }
}
