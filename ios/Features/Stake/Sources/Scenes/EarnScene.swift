// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemEarnSection
import struct Gemstone.GemEarnView
import Primitives
import PrimitivesComponents
import SwiftUI

public struct EarnScene: View {
    private let model: EarnSceneViewModel

    public init(model: EarnSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        let earn = model.earnView
        List {
            ListAssetHeaderView(model: earn.asset)

            Section {
                GemListRowView(row: earn.rateRow)
            }

            ForEach(earn.sections, id: \.self) { section in
                Section(section.title) {
                    content(for: section, earn: earn)
                }
            }

            if earn.showsEmpty {
                Section {
                    EmptyContentView(model: model.emptyContentModel)
                        .cleanListRow()
                }
            }
        }
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .refreshable {
            await model.load()
        }
        .taskOnce {
            Task {
                await model.load()
            }
        }
    }
}

// MARK: - UI Components

extension EarnScene {
    @ViewBuilder
    private func content(for section: GemEarnSection, earn: GemEarnView) -> some View {
        switch section {
        case .manage:
            NavigationCustomLink(with: GemListRowView(row: earn.depositRow)) {
                model.onSelectDeposit()
            }
        case .positions:
            ForEach(earn.positions) { item in
                NavigationCustomLink(with: ListItemView(model: item.row.listItem)) {
                    model.onSelect(item: item)
                }
            }
            .listRowInsets(.assetListRowInsets)
        }
    }
}
