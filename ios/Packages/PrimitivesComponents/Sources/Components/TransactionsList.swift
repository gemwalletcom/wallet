// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct TransactionsList: View {
    private let sections: [ListSection<TransactionViewModel>]
    private let currency: String

    public init(sections: [ListSection<TransactionViewModel>], currency: String) {
        self.sections = sections
        self.currency = currency
    }

    public var body: some View {
        ForEach(sections) { section in
            Section {
                ForEach(section.values) { model in
                    NavigationLink(value: Scenes.Transaction(transaction: model.transaction)) {
                        TransactionView(model: model, currency: currency)
                    }
                }
            } header: {
                section.title.map { Text($0) }
            }
        }
    }
}
