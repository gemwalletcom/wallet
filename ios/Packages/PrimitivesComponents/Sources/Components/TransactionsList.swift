// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemTransactionRow
import GemstonePrimitives
import Primitives
import SwiftUI

public struct TransactionsList: View {
    private let sections: [ListSection<GemTransactionRow>]
    private let onEndReached: () -> Void

    public init(sections: [ListSection<GemTransactionRow>], onEndReached: @escaping () -> Void = {}) {
        self.sections = sections
        self.onEndReached = onEndReached
    }

    public var body: some View {
        let endRowIds = sections.lastRowIds(GemConstants.listPrefetchRows)
        ForEach(sections) { section in
            Section {
                ForEach(section.values) { row in
                    NavigationLink(value: Scenes.Transaction(id: row.id)) {
                        TransactionView(row: row)
                    }
                    .accessibilityIdentifier("transaction")
                    .onAppear {
                        if endRowIds.contains(row.id) {
                            onEndReached()
                        }
                    }
                }
            } header: {
                section.title.map { Text($0) }
            }
        }
    }
}
