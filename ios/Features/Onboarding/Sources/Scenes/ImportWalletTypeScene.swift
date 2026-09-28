// Copyright (c). Gem Wallet. All rights reserved.

import Components
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

struct ImportWalletTypeScene: View {
    let model: ImportWalletTypeSceneViewModel
    @State private var searchQuery = ""

    init(
        model: ImportWalletTypeSceneViewModel,
    ) {
        self.model = model
    }

    var body: some View {
        let types = model.types(for: searchQuery)
        return List {
            Section {
                NavigationLink(value: ImportWalletType.multicoin) {
                    ListItemView(model: model.multicoinListItem(types))
                }
            }

            if types.chains.isEmpty {
                StateEmptyView(title: Localized.Common.noResultsFound)
            } else {
                Section {
                    ForEach(types.chains, id: \.chain) { row in
                        NavigationLink(value: ImportWalletType.chain(Chain(core: row.chain))) {
                            ChainView(model: row)
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
