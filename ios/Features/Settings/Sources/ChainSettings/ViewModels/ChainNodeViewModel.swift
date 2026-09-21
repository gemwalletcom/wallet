// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNodeRow
import struct Gemstone.GemNodeSelection
import Localization
import PrimitivesComponents

struct ChainNodeViewModel {
    let row: GemNodeRow

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

    var listItem: ListItemModel {
        row.latencyStatus.listItem(
            title: row.title.text(gemNodeLabel: Localized.Nodes.gemWalletNode),
            titleExtra: row.subtitle.text,
        )
    }
}

// MARK: - Identifiable

extension ChainNodeViewModel: Identifiable {
    var id: String {
        row.node.url
    }
}
