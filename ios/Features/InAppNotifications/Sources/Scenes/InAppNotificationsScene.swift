// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemNotificationRow
import Localization
import PrimitivesComponents
import Store
import SwiftUI

public struct InAppNotificationsScene: View {
    @State private var model: InAppNotificationsSceneViewModel

    public init(model: InAppNotificationsSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        let sections = model.sections
        return List {
            if let error = model.loadError {
                Section {
                    ListItemErrorView(errorTitle: Localized.Errors.errorOccurred, error: error)
                }
            }
            ForEach(sections) { section in
                Section(header: section.title.map { Text($0) }) {
                    ForEach(section.values) { row in
                        notificationRow(row)
                    }
                }
            }
            .listRowInsets(.assetListRowInsets)
        }
        .listSectionSpacing(.compact)
        .overlay {
            if sections.isEmpty, model.loadError == nil {
                EmptyContentView(model: model.emptyContentModel)
            }
        }
        .navigationTitle(model.title)
        .bindQuery(model.query)
        .task { await model.load() }
    }

    @ViewBuilder
    private func notificationRow(_ row: GemNotificationRow) -> some View {
        let view = ListItemView(model: row.listItem)
        if let destination = row.destination {
            NavigationCustomLink(with: view) {
                model.open(destination: destination)
            }
        } else {
            view
        }
    }
}
