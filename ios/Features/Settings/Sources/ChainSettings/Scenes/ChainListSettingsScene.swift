// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct ChainListSettingsScene: View {
    @State private var model: ChainListSettingsViewModel
    @State private var searchQuery = ""

    public init(model: ChainListSettingsViewModel = ChainListSettingsViewModel()) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            Section {
                NavigationLink(value: Scenes.ServiceStatus()) {
                    ListItemView(
                        title: Localized.Transaction.status,
                        imageStyle: .asset(assetImage: AssetImage.image(Images.Logo.logo)),
                    )
                }
            }

            Section("Chains") {
                ForEach(filteredChains) { chain in
                    NavigationLink(value: Scenes.ChainSettings(chain: chain)) {
                        ChainView(model: ChainViewModel(chain: chain))
                    }
                }
            }
        }
        .listRowInsets(.assetListRowInsets)
        .listSectionSpacing(.compact)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .searchable(
            text: $searchQuery,
            placement: .navigationBarDrawer(displayMode: .always),
        )
        .autocorrectionDisabled(true)
        .scrollDismissesKeyboard(.interactively)
        .overlay {
            if filteredChains.isEmpty {
                ContentUnavailableView {
                    EmptyContentView(model: model.emptyContent)
                }
                .background(UIColor.systemGroupedBackground.color)
            }
        }
        .navigationTitle(Localized.Settings.Networks.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Private

extension ChainListSettingsScene {
    private var filteredChains: [Chain] {
        model.filterChains(for: searchQuery)
    }
}
