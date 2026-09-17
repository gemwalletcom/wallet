// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct ImportWalletTypeScene: View {
    let model: ImportWalletTypeViewModel
    @State private var searchQuery = ""

    init(
        model: ImportWalletTypeViewModel,
    ) {
        self.model = model
    }

    var body: some View {
        List {
            Section {
                NavigationLink(value: ImportWalletType.multicoin) {
                    ListItemView(model: model.multicoinListItem)
                }
            }

            if model.items(for: searchQuery).isEmpty {
                StateEmptyView(title: Localized.Common.noResultsFound)
            } else {
                Section {
                    ForEach(model.items(for: searchQuery)) { chain in
                        NavigationLink(value: ImportWalletType.chain(chain)) {
                            ListItemView(model: model.listItem(for: chain))
                        }
                    }
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .navigationBarTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarInfoButton(url: AppUrl.docs(.migrateWallet))
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .always),
        )
        .autocorrectionDisabled(true)
        .scrollDismissesKeyboard(.interactively)
    }
}
