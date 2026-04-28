// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
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
                ForEach(model.connectivityModels) { item in
                    ListItemView(
                        title: item.title,
                        titleTag: item.titleTag,
                        titleTagStyle: item.titleTagStyle,
                        titleTagType: item.titleTagType,
                        titleExtra: item.subtitle,
                    )
                }
            }

            Section(model.chainsTitle) {
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
        .refreshable {
            await model.fetchConnectivity()
        }
        .taskOnce {
            Task { await model.fetchConnectivity() }
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}

// MARK: - Private

extension ChainListSettingsScene {
    private var filteredChains: [Chain] {
        guard !searchQuery.isEmpty else {
            return model.chains
        }

        return model.chains.filter {
            model.filter($0, query: searchQuery)
        }
    }
}
