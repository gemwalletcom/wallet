// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SecurityScene: View {
    @State private var model: SecuritySceneViewModel

    public init(model: SecuritySceneViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { row in
            if case .picker = row {
                GemListRowView(row: row, onSelect: model.onSelect)
                    .confirmationDialog(model.lockPeriodTitle, isPresented: $model.isPresentingLockPeriods) {
                        ForEach(model.allLockPeriods) { period in
                            Button(period.title) { model.updateLockPeriod(to: period) }
                        }
                    }
            } else {
                GemListRowView(row: row, onToggle: model.onToggle)
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
