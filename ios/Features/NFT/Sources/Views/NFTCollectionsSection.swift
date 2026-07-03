// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import SwiftUI

public struct NFTCollectionsSection: View {
    private let model: CollectionsViewModel

    public init(model: CollectionsViewModel) {
        self.model = model
    }

    public var body: some View {
        if model.isEmpty {
            Section {
                EmptyContentView(model: model.emptyContentModel)
                    .padding(.vertical, .extraLarge)
            }
            .cleanListRow()
        } else {
            Section {} header: {
                LazyVGrid(columns: model.columns) {
                    ForEach(model.content.items) { item in
                        NavigationLink(value: item.destination) {
                            GridPosterView(model: item.model)
                        }
                    }
                }
            }
            .cleanListRow()
            .listSectionSpacing(.custom(.medium))

            if let unverifiedCount = model.unverifiedCount {
                Section {
                    NavigationLink(value: Scenes.UnverifiedCollections()) {
                        ListItemView(
                            title: Localized.Asset.Verification.unverified,
                            subtitle: unverifiedCount,
                        )
                    }
                }
                .listRowInsets(.assetListRowInsets)
                .listSectionSpacing(.custom(.medium))
            }
        }
    }
}
