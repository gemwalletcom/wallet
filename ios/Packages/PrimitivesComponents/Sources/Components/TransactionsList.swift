// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemTransactionRow
import Primitives
import SwiftUI

public struct TransactionsList: View {
    private let sections: [ListSection<GemTransactionRow>]

    public init(sections: [ListSection<GemTransactionRow>]) {
        self.sections = sections
    }

    public var body: some View {
        ForEach(sections) { section in
            Section {
                ForEach(section.values) { row in
                    NavigationLink(value: Scenes.Transaction(id: row.transactionId)) {
                        TransactionView(row: row)
                    }
                }
            } header: {
                section.title.map { Text($0) }
            }
        }
    }
}
