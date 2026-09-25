// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Localization
import Primitives
import PrimitivesComponents
import Store
import Style
import SwiftUI

public struct CollectionsScene: View {
    @State private var model: CollectionsViewModel

    public init(model: CollectionsViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let screen = model.screen
        let content = CollectionsContent(screen)
        return GeometryReader { geometry in
            ScrollView {
                VStack(spacing: .zero) {
                    if content.items.isNotEmpty {
                        LazyVGrid(columns: model.columns) {
                            collectionsView(content.items)
                        }
                        .padding(.horizontal, Spacing.medium + Spacing.tiny)

                        Spacer(minLength: .medium)
                    }

                    if let unverifiedItem = content.unverifiedListItem {
                        List {
                            NavigationLink(value: Scenes.UnverifiedCollections()) {
                                ListItemView(model: unverifiedItem)
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
    private func collectionsView(_ items: [GridPosterViewItem]) -> some View {
        ForEach(items) { item in
            NavigationLink(value: item.destination) {
                GridPosterView(model: item.model)
            }
        }
    }
}
