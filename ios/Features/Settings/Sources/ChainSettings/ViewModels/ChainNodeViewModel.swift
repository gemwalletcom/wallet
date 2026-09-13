// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import enum Gemstone.GemNodeRowTitle
import struct Gemstone.GemNodeSelection
import enum Gemstone.GemNodeStatusState
import Localization
import Style

struct ChainNodeViewModel {
    let node: GemNodeSelection

    private let statusState: GemNodeStatusState
    private let formatter: ValueFormatter

    init(
        node: GemNodeSelection,
        statusState: GemNodeStatusState,
        formatter: ValueFormatter,
    ) {
        self.node = node
        self.statusState = statusState
        self.formatter = formatter
    }

    var url: String {
        node.url
    }

    var selection: String? {
        node.isSelected ? node.url : .none
    }

    var title: String {
        switch node.title() {
        case let .host(host): host
        case let .gemNode(flag): Localized.Nodes.gemWalletNode + " " + flag
        }
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
        node.url
    }
}
