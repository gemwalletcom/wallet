// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct SimulationPayloadDetailsScene: View {
    @Environment(\.dismiss) private var dismiss

    private let primaryModels: [SimulationPayloadFieldViewModel]
    private let secondaryModels: [SimulationPayloadFieldViewModel]
    private let actionListItem: ListItemModel?
    private let actionDestination: AnyView?

    public init(
        primaryModels: [SimulationPayloadFieldViewModel],
        secondaryModels: [SimulationPayloadFieldViewModel],
        actionListItem: ListItemModel? = nil,
        actionDestination: AnyView? = nil,
    ) {
        self.primaryModels = primaryModels
        self.secondaryModels = secondaryModels
        self.actionListItem = actionListItem
        self.actionDestination = actionDestination
    }

    public var body: some View {
        List {
            if !primaryModels.isEmpty {
                Section {
                    SimulationPayloadFieldsContent(models: primaryModels)
                }
            }

            if !secondaryModels.isEmpty {
                Section(Localized.Common.details) {
                    SimulationPayloadFieldsContent(models: secondaryModels)
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
}
