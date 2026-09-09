// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionDetailRow
import struct Gemstone.GemTransactionDetailSection
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

extension GemTransactionDetailRow: @retroactive Identifiable {
    public var id: Self {
        self
    }
}

public extension ListSection where T == GemTransactionDetailRow {
    init(_ section: GemTransactionDetailSection) {
        self.init(id: "\(section.rows[0])", title: nil, image: nil, values: section.rows)
    }
}

public enum TransactionItemModel {
    case listItem(ListItemModel)
    case fee(ListItemModel)
    case header(TransactionHeaderItemModel)
    case swapProgress(TransactionSwapProgressItemModel)
    case participant(TransactionParticipantItemModel)
    case rate(title: String, value: String)
    case network(title: String, subtitle: String, image: AssetImage)
    case pnl(title: String, value: String, color: Color)
    case price(title: String, value: String)
    case explorer(url: URL, text: String)
    case swapAgain(text: String)
    case empty
}
