// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import struct Gemstone.GemNodeRow
import struct Gemstone.GemNodeSelection
import Localization
import Style

struct ChainNodeViewModel {
    let row: GemNodeRow

    init(row: GemNodeRow) {
        self.row = row
    }

    var node: GemNodeSelection {
        row.node
    }

    var url: String {
        row.node.url
    }

    var canDelete: Bool {
        row.canDelete
    }

    var selection: String? {
        row.node.isSelected ? row.node.url : .none
    }

    var title: String {
        row.title.text(gemNodeLabel: Localized.Nodes.gemWalletNode)
    }

    var titleExtra: String? {
        row.subtitle.text(latestBlockLabel: row.subtitle.title)
    }

    var titleTag: String? {
        statusTag.text
    }

    var titleTagType: TitleTagType {
        statusTag.type
    }

    var titleTagStyle: TextStyle {
        statusTag.style
    }

    private var statusTag: LatencyStatusViewModel {
        LatencyStatusViewModel(status: row.latencyStatus)
    }
}

// MARK: - Identifiable

extension ChainNodeViewModel: Identifiable {
    var id: String {
        row.node.url
    }
}
