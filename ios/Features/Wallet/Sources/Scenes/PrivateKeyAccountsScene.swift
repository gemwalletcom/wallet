// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI

struct PrivateKeyAccountsScene: View {
    private let model: PrivateKeyExportViewModel

    init(model: PrivateKeyExportViewModel) {
        self.model = model
    }

    var body: some View {
        List {
            Section {
                HStack(alignment: .top, spacing: .small) {
                    Images.System.info
                        .foregroundStyle(Colors.blue)
                    Text(model.accountsInfo)
                        .textStyle(.calloutSecondary)
                }
            }

            Section {
                ForEach(model.accounts, id: \.row.chain) { account in
                    NavigationCustomLink(with: ListItemView(model: model.listItem(for: account))) {
                        Task { await model.onSelect(chain: account.row.chain) }
                    }
                }
            } header: {
                Text(model.selectAccountTitle)
            }
        }
        .contentMargins([.top], .extraSmall, for: .scrollContent)
        .navigationTitle(model.title)
        .toolbarTitleDisplayMode(.inline)
        .task { await model.fetchAccounts() }
    }
}
