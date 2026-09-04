// Copyright (c). Gem Wallet. All rights reserved.

import BigInt
import Components
import Formatters
import Foundation
import struct Gemstone.GemNodeCheck
import GemstonePrimitives
import Localization
import Primitives
import Style

struct AddNodeResultViewModel {
    static let valueFormatter = ValueFormatter.full_US

    private let result: GemNodeCheck

    init(result: GemNodeCheck) {
        self.result = result
    }

    var url: String {
        result.url
    }

    var chainIdField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.chainId, value: result.chainId ?? Placeholder.empty)
    }

    var inSyncField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.inSync, value: result.isInSync ? Emoji.checkmark : Emoji.reject)
    }

    var latestBlockField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.latestBlock, value: Self.valueFormatter.string(BigInt(result.latestBlockNumber), decimals: 0))
    }

    var latencyField: ListItemField {
        ListItemField(title: Localized.Nodes.ImportNode.latency, value: LatencyViewModel(latency: result.latency.map()).title)
    }
}
