// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import SwiftUI

public struct TransactionsList: View {
    private let sections: [ListSection<TransactionViewModel>]

    public init(sections: [ListSection<TransactionViewModel>]) {
        self.sections = sections
    }

    public var body: some View {
        ForEach(sections) { section in
            Section {
                ForEach(section.values) { model in
                    NavigationLink(value: Scenes.Transaction(id: model.transactionId)) {
                        TransactionView(model: model)
                    }
                }
            } header: {
                section.title.map { Text($0) }
            }
        }
    }
}
