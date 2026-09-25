// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct SecurityScene: View {
    @State private var model: SecurityViewModel

    public init(model: SecurityViewModel) {
        self.model = model
    }

    public var body: some View {
        ListSectionView(sections: model.sections) { row in
            GemListRowView(row: row, onToggle: model.onToggle, onSelect: model.onSelect)
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .confirmationDialog(model.lockPeriodTitle, isPresented: $model.isPresentingLockPeriods) {
            ForEach(model.allLockPeriods) { period in
                Button(period.title) { model.updateLockPeriod(to: period) }
            }
        }
        .alertSheet($model.isPresentingAlertMessage)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }
}
