// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import struct Gemstone.GemNftEntry
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct CollectionsScene: View {
    @State private var model: CollectionsSceneViewModel

    public init(model: CollectionsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let screen = model.screen
        return GeometryReader { geometry in
            ScrollView {
                VStack(spacing: .zero) {
                    if screen.items.isNotEmpty {
                        LazyVGrid(columns: model.columns) {
                            collectionsView(screen.items)
                        }
                        .padding(.horizontal, Spacing.medium + Spacing.tiny)

                        Spacer(minLength: .medium)
                    }

                    if let unverifiedRow = screen.unverifiedRow {
                        List {
                            NavigationLink(value: Scenes.UnverifiedCollections()) {
                                ListItemView(model: unverifiedRow.listItem)
                            }
                        }
                        .contentMargins(.top, .zero, for: .scrollContent)
                        .scrollDisabled(true)
                        .frame(height: .list.minHeight)
                    }
                }
                .frame(
                    maxWidth: .infinity,
                    minHeight: geometry.size.height - Spacing.scene.top,
                    alignment: .top,
                )
            }
            .scrollIndicators(
                screen.hasContent ? .automatic : .hidden,
            )
        }
        .bindQuery(model.query)
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .overlay {
            if let error = model.loadError(screen) {
                ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                    .padding(.horizontal, .medium)
            } else if !screen.hasContent {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
        .background { Colors.insetGroupedListStyle.ignoresSafeArea() }
        .navigationBarTitleDisplayMode(.inline)
        .navigationTitle(screen.title.text)
        .refreshable { await model.load() }
        .toast(message: $model.isPresentingToastMessage)
        .task {
            guard screen.syncsOnAppear else { return }
            await model.load()
        }
    }
}

// MARK: - UI

extension CollectionsScene {
    private func collectionsView(_ entries: [GemNftEntry]) -> some View {
        ForEach(entries, id: \.row.id) { entry in
            NavigationLink(value: entry.destination) {
                GridPosterView(model: entry.posterModel)
            }
        }
    }
}
