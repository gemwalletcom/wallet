// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemListRow
import Localization
import Style
import SwiftUI

public struct SimulationPayloadDetailsScene: View {
    @Environment(\.dismiss) private var dismiss

    private let primaryRows: [GemListRow]
    private let secondaryRows: [GemListRow]
    private let onSelectAddress: (String) -> Void
    private let actionListItem: ListItemModel?
    private let actionDestination: AnyView?

    public init(
        primaryRows: [GemListRow],
        secondaryRows: [GemListRow],
        onSelectAddress: @escaping (String) -> Void,
        actionListItem: ListItemModel? = nil,
        actionDestination: AnyView? = nil,
    ) {
        self.primaryRows = primaryRows
        self.secondaryRows = secondaryRows
        self.onSelectAddress = onSelectAddress
        self.actionListItem = actionListItem
        self.actionDestination = actionDestination
    }

    public var body: some View {
        List {
            if !primaryRows.isEmpty {
                Section {
                    rows(primaryRows)
                }
            }

            if !secondaryRows.isEmpty {
                Section(Localized.Common.details) {
                    rows(secondaryRows)
                }
            }

            if let actionListItem, let actionDestination {
                Section {
                    NavigationLink {
                        actionDestination
                    } label: {
                        ListItemView(model: actionListItem)
                    }
                }
            }
        }
        .toolbar {
            ToolbarItem(placement: .topBarTrailing) {
                Button("", systemImage: SystemImage.checkmark, action: { dismiss() })
            }
        }
        .navigationTitle(Localized.Common.details)
        .navigationBarTitleDisplayMode(.inline)
        .listStyle(.insetGrouped)
        .listRowSpacing(.zero)
        .listSectionSpacing(.compact)
    }

    private func rows(_ rows: [GemListRow]) -> some View {
        ForEach(Array(rows.enumerated()), id: \.offset) {
            GemListRowView(row: $0.element, onSelectAddress: onSelectAddress)
        }
    }
}
