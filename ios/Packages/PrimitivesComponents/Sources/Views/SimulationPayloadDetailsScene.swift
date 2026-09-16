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
    private let actionTitle: String?
    private let actionDestination: AnyView?

    public init(
        primaryModels: [SimulationPayloadFieldViewModel],
        secondaryModels: [SimulationPayloadFieldViewModel],
        actionTitle: String? = nil,
        actionDestination: AnyView? = nil,
    ) {
        self.primaryModels = primaryModels
        self.secondaryModels = secondaryModels
        self.actionTitle = actionTitle
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

            if let actionTitle, let actionDestination {
                Section {
                    NavigationLink {
                        actionDestination
                    } label: {
                        ListItemView(title: actionTitle)
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
