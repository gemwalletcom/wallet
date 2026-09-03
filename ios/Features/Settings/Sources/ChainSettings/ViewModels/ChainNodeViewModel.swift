// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import struct Gemstone.GemNodeSelection
import enum Gemstone.GemNodeStatusState
import Localization
import Style

struct ChainNodeViewModel {
    let node: GemNodeSelection

    private let gemNodeFlag: String?
    private let statusState: GemNodeStatusState
    private let formatter: ValueFormatter

    init(
        node: GemNodeSelection,
        gemNodeFlag: String?,
        statusState: GemNodeStatusState,
        formatter: ValueFormatter,
    ) {
        self.node = node
        self.gemNodeFlag = gemNodeFlag
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
        guard let gemNodeFlag else { return node.host }
        return Localized.Nodes.gemWalletNode + " " + gemNodeFlag
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
