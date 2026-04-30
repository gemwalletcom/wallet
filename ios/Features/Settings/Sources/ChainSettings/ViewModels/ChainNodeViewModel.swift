// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import Localization
import Primitives
import Style

struct ChainNodeViewModel {
    let chainNode: ChainNode

    private let statusState: NodeStatusState
    private let formatter: ValueFormatter

    init(
        chainNode: ChainNode,
        statusState: NodeStatusState,
        formatter: ValueFormatter,
    ) {
        self.chainNode = chainNode
        self.statusState = statusState
        self.formatter = formatter
    }

    var title: String {
        guard let host = chainNode.host else { return "" }

        let flag: String? = switch host {
        case Constants.nodesAsiaURL.cleanHost(): "🇯🇵"
        case Constants.nodesEuropeURL.cleanHost(): "🇪🇺"
        case Constants.nodesURL.cleanHost(): "🇺🇸"
        default: nil
        }

        if let flag {
            return Localized.Nodes.gemWalletNode + " " + flag
        }
        return host
    }

    var titleExtra: String? {
        nodeStatusModel
            .latestBlockText(
                title: Localized.Nodes.ImportNode.latestBlock,
                formatter: formatter,
            )
    }

    var titleTag: String? {
        nodeStatusModel.latencyText
    }

    var titleTagType: TitleTagType {
        nodeStatusModel.titleTagType
    }

    var titleTagStyle: TextStyle {
        nodeStatusModel.titleTagStyle
    }

    private var nodeStatusModel: NodeStatusStateViewModel {
        NodeStatusStateViewModel(nodeStatus: statusState)
    }
}

// MARK: - Identifiable

extension ChainNodeViewModel: Identifiable {
    var id: String {
        chainNode.id
    }
}

// MARK: - Models extensions

extension ChainNode {
    var host: String? {
        URL(string: node.url)?.host
    }

    var isGemNode: Bool {
        host?.contains("gemnodes.com") ?? false
    }
}
