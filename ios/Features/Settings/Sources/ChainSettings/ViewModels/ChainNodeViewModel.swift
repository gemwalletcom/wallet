// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Formatters
import Foundation
import enum Gemstone.GemNodeStatusState
import GemstonePrimitives
import Localization
import Primitives
import Style

struct ChainNodeViewModel {
    let chainNode: ChainNode

    private let gemNodeFlag: String?
    private let statusState: GemNodeStatusState
    private let formatter: ValueFormatter

    init(
        chainNode: ChainNode,
        gemNodeFlag: String?,
        statusState: GemNodeStatusState,
        formatter: ValueFormatter,
    ) {
        self.chainNode = chainNode
        self.gemNodeFlag = gemNodeFlag
        self.statusState = statusState
        self.formatter = formatter
    }

    var title: String {
        guard let host = chainNode.host else { return "" }
        guard let gemNodeFlag else { return host }
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
        chainNode.id
    }
}

// MARK: - Models extensions

extension ChainNode {
    var host: String? {
        URL(string: node.url)?.host
    }
}
