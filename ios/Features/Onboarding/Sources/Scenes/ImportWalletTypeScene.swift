// Copyright (c). Gem Wallet. All rights reserved.

import Components
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
                    ListItemView(
                        title: Localized.Wallet.multicoin,
                        imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)),
                    )
                }
            }

            if model.items(for: searchQuery).isEmpty {
                StateEmptyView(title: Localized.Common.noResultsFound)
            } else {
                Section {
                    ForEach(model.items(for: searchQuery)) { chain in
                        NavigationLink(value: ImportWalletType.chain(chain)) {
                            ListItemView(
                                title: Asset(chain).name,
                                imageStyle: .asset(assetImage: AssetImage.image(ChainImage(chain: chain).placeholder)),
                            )
                        }
                    }
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .navigationBarTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .always),
        )
        .autocorrectionDisabled(true)
        .scrollDismissesKeyboard(.interactively)
    }
}
